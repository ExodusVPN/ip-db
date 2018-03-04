extern crate futures;
extern crate tokio_core;
extern crate flate2;
extern crate hyper;
extern crate hyper_tls;


use self::futures::{Future};
use self::tokio_core::reactor;
use self::hyper::Uri;


use std::time::Duration;
use std::sync::mpsc::{ channel, Sender };
use std::path::PathBuf;
use std::io::{Write, Read};
use std::fs::{self, File, OpenOptions};


#[derive(Debug)]
pub enum Protocol {
    Http,
    Https
}

#[derive(Debug, Clone)]
pub enum State {
    Init,
    UpdateMd5File,
    UpdateFile,
    Success,
    Failure(String),
}

#[derive(Debug)]
pub struct Task {
    index: usize,
    fileuri: Uri,
    filepath: PathBuf,
    md5_fileuri: Uri,
    md5_filepath: PathBuf,
    protocol: Protocol,
    state: State,
    lock: bool,
}


impl Task {
    pub fn run(&mut self, handler: &reactor::Handle, tx: Sender<(usize, State)>) {
        if self.lock {
            return ();
        }

        match self.state {
            State::Init => {
                tx.send((self.index, State::UpdateMd5File)).unwrap();
                self.lock = true;
            },
            State::UpdateMd5File => {
                match self.protocol {
                    Protocol::Http | Protocol::Https => {
                        let https_client = hyper::Client::configure()
                            .connector(hyper_tls::HttpsConnector::new(2, &handler).unwrap())
                            .build(&handler);

                        let md5_fileuri = self.md5_fileuri.clone();
                        let md5_filepath = self.md5_filepath.clone();
                        let md5_filepath2 = self.md5_filepath.clone();
                        let task_index = self.index;
                        let tx2 = tx.clone();

                        handler.spawn(
                            Box::new(https_client.get(md5_fileuri.clone())
                                .and_then(move |res| {
                                    use self::futures::Stream;
                                    let status_code = res.status().as_u16();

                                    res.body()
                                        .concat2()
                                        .map(|chunk|{
                                            chunk.to_vec()
                                        })
                                        .and_then(move |bytes|{
                                            if status_code != 200 {
                                                error!("GET {:?}    StatusCode: {}", md5_fileuri, status_code);
                                                Err(hyper::Error::Status)
                                            } else {
                                                String::from_utf8(bytes)
                                                    .map_err(|e| e.into())
                                            }
                                        })
                                })
                                .map(move |body: String|{
                                    if !md5_filepath.exists() {
                                        File::create(&md5_filepath).unwrap();
                                    }

                                    let local_md5_file_content = {
                                        let mut local_file = File::open(&md5_filepath).unwrap();
                                        let mut local_md5_file_content = String::new();
                                        local_file.read_to_string(&mut local_md5_file_content).unwrap();
                                        drop(local_file);
                                        local_md5_file_content
                                    };

                                    let need_update = body != local_md5_file_content || body == "".to_string();

                                    if need_update {
                                        // Need Update
                                        let mut file = OpenOptions::new().create(false).write(true).append(false)
                                            .open(&md5_filepath).unwrap();
                                        file.write_all(&body.as_bytes()).unwrap();
                                        tx.send((task_index, State::UpdateFile)).unwrap();
                                    } else {
                                        // Is Up-to-date
                                        trace!("File {:?} Is Up-to-date!  ({:?})  => ({:?})", md5_filepath, body, local_md5_file_content);
                                        tx.send((task_index, State::Success)).unwrap();
                                    }
                                })
                                .map_err(move |e: hyper::Error|{
                                    fs::remove_file(md5_filepath2).unwrap();
                                    tx2.send((task_index, State::Failure( format!("{}", e) ))).unwrap();
                                })
                            )
                        );
                        self.lock = true;
                    },
                }
            },
            State::UpdateFile => {
                match self.protocol {
                    Protocol::Http | Protocol::Https => {
                        let https_client = hyper::Client::configure()
                            .connector(hyper_tls::HttpsConnector::new(2, &handler).unwrap())
                            .build(&handler);

                        let fileuri = self.fileuri.clone();
                        let filepath = self.filepath.clone();
                        let md5_filepath = self.md5_filepath.clone();

                        let task_index = self.index;
                        let tx2 = tx.clone();

                        handler.spawn(
                            Box::new(https_client.get(fileuri.clone())
                                .and_then(move |res| {
                                    let status_code = res.status().as_u16();
                                    use self::futures::Stream;

                                    res.body()
                                        .concat2()
                                        .map(|chunk| {
                                            chunk.to_vec()
                                        })
                                        .and_then(move |bytes| {
                                            if status_code != 200 {
                                                error!("GET {:?}    StatusCode: {}", fileuri, status_code);
                                                Err(hyper::Error::Status)
                                            } else {
                                                Ok(bytes)
                                            }
                                        })
                                })
                                .map(move |body: Vec<u8>|{
                                    let mut file = OpenOptions::new().create(true).write(true).append(false)
                                                .open(&filepath).unwrap();
                                    trace!("GET {:?}    Bytes: {}", filepath, body.len());

                                    file.write_all(&body).unwrap();
                                    tx.send((task_index, State::Success)).unwrap();
                                    ()
                                })
                                .map_err(move |e: hyper::Error|{
                                    fs::remove_file(md5_filepath).unwrap();
                                    tx2.send((task_index, State::Failure( format!("{}", e) ))).unwrap();
                                    ()
                                })
                            )
                        );
                        self.lock = true;
                    },
                }
            },
            State::Success | State::Failure(_) => {

            },
        }
    }
}



pub fn sync(data_path: PathBuf) {
    let mut core = reactor::Core::new().unwrap();
    
    let urls = [
        //// ("delegated-arin-latest",             "https://ftp.arin.net/pub/stats/arin/delegated-arin-latest"),
        ("delegated-arin-extended-latest",    "https://ftp.arin.net/pub/stats/arin/delegated-arin-extended-latest"),
        ("delegated-ripencc-latest",          "https://ftp.ripe.net/pub/stats/ripencc/delegated-ripencc-latest"),
        ("delegated-ripencc-extended-latest", "https://ftp.ripe.net/pub/stats/ripencc/delegated-ripencc-extended-latest"),
        ("delegated-apnic-latest",            "https://ftp.apnic.net/stats/apnic/delegated-apnic-latest"),
        ("delegated-apnic-extended-latest",   "https://ftp.apnic.net/stats/apnic/delegated-apnic-extended-latest"),
        ("delegated-lacnic-latest",           "http://ftp.lacnic.net/pub/stats/lacnic/delegated-lacnic-latest"),
        ("delegated-lacnic-extended-latest",  "http://ftp.lacnic.net/pub/stats/lacnic/delegated-lacnic-extended-latest"),
        ("delegated-afrinic-latest",          "https://ftp.afrinic.net/pub/stats/afrinic/delegated-afrinic-latest"),
        ("delegated-afrinic-extended-latest", "https://ftp.afrinic.net/pub/stats/afrinic/delegated-afrinic-extended-latest"),
        ("delegated-iana-latest",             "https://ftp.apnic.net/stats/iana/delegated-iana-latest"),
        //// ("delegated-iana-extended-latest",    "ftp://ftp.apnic.net/public/stats/iana/delegated-iana-extended-latest"),
    ];
    
    let mut tasks: Vec<Task> = (0..urls.len()).map(|idx|{
        let filename = urls[idx].0;
        let url = urls[idx].1;
        let fileuri = url.parse::<Uri>().unwrap();
        let protocol = match fileuri.scheme() {
            Some(scheme) => match scheme {
                "http" => Protocol::Http,
                "https" => Protocol::Https,
                _ => panic!("URL Scheme Not Supported.")
            }
            None => panic!("Unknow URL Scheme.")
        };

        Task {
            index: idx,
            filepath    : data_path.join(filename),
            md5_filepath: data_path.join(format!("{}.md5", filename)),
            fileuri     : fileuri,
            md5_fileuri : (format!("{}.md5", url)).parse::<Uri>().unwrap(),
            protocol: protocol,
            state: if filename == "delegated-iana-latest" { State::UpdateFile } else { State::Init },
            lock: false,
        }
    }).collect();

    let (tx, rx) = channel::<(usize, State)>();
    let timeout = Duration::new(2, 0);


    info!("\nTasks:\n{}", tasks.iter()
        .map(|task|{
            format!("\t{:50} {:?}", format!("{:?}", task.filepath), task.fileuri)
        })
        .collect::<Vec<String>>()
        .join("\n"));
    

    loop {
        let mut sum = 0;
        
        for task in tasks.iter_mut() {
            match task.state {
                State::Success | State::Failure(_) => {
                    sum += 1;
                },
                _ => {
                    if !task.lock {
                        task.run(&core.handle(), tx.clone());
                    }
                },
            }
        }

        if sum == tasks.len() {
            break
        }

        core.turn(Some(timeout));

        if let Ok((index, state)) = rx.try_recv() {
            tasks[index].state = state;
            tasks[index].lock = false;

            info!("Event:\tFile: {:50} State: {:?}", format!("{:?}", tasks[index].filepath), tasks[index].state);
        }
    }
}

