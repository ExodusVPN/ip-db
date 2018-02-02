
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate ftp;
extern crate tokio_core;
extern crate smoltcp;
extern crate clap;

extern crate ip_db;


use futures::{Future};
use futures::future;
use tokio_core::reactor::Core;
#[allow(unused_imports)]
use smoltcp::wire::{
    IpAddress, IpCidr, Ipv4Address, Ipv4Cidr, 
    Ipv6Address, Ipv6Cidr
};


use std::fmt;
use std::time::Duration;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::io::{Write, Read, Cursor};
use std::fs::{self, File, OpenOptions};
use std::net::{ToSocketAddrs, Ipv4Addr, Ipv6Addr};

use ip_db::{Registry, Country, Status};


const DATA_DIR: &'static str = "data";


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
struct Ipv4Record {
    src_registry: Registry,
    country: Country,
    start: Ipv4Address,
    num: usize,
    status: Status,
    dst_registry: Option<Registry>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
struct Ipv6Record {
    src_registry: Registry,
    country: Country,
    start: Ipv6Address,
    prefix: u8,
    status: Status,
    dst_registry: Option<Registry>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
enum Record {
    Ipv4(Ipv4Record),
    Ipv6(Ipv6Record)
}


impl fmt::Display for Ipv4Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} ipv4 {} {} {} {}",
            self.src_registry,
            self.country,
            format!("{}", self.start),
            self.num,
            self.status,
            match self.dst_registry {
                Some(reg) => format!("{}", reg),
                None => "none".to_string()
            })
    }
}

impl fmt::Display for Ipv6Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} ipv6 {} {} {} {}",
            self.src_registry,
            self.country,
            format!("{}", self.start),
            self.prefix,
            self.status,
            match self.dst_registry {
                Some(reg) => format!("{}", reg),
                None => "none".to_string()
            })
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Record::Ipv4(ref ipv4_record) => fmt::Display::fmt(&ipv4_record, f),
            &Record::Ipv6(ref ipv6_record) => fmt::Display::fmt(&ipv6_record, f),
        }
    }
}



fn get_data_dir() -> PathBuf {
    let path = Path::new(DATA_DIR);
    if path.exists() == false {
        fs::create_dir(path).unwrap();
    }
    path.to_path_buf()
}

fn ftp_get<W: Write, A: ToSocketAddrs>(sa: &A, filepath: &str, output: &mut W) -> Result<usize, ftp::types::FtpError> {
    let mut ftp_buffer = [0u8; 4096];
    let mut size = 0usize;
    match ftp::FtpStream::connect(sa) {
        Ok(mut ftp_stream) => {
            ftp_stream.login("anonymous", "anonymous@rust-lang.org").unwrap();
            let ret = match ftp_stream.get(filepath) {
                Ok(mut buf_reader) => {
                    loop {
                        match buf_reader.read(&mut ftp_buffer) {
                            Ok(amt) => {
                                if amt == 0 {
                                    break;
                                }
                                output.write(&ftp_buffer[..amt]).unwrap();
                                size += amt;
                            }
                            Err(e) => return Err(ftp::FtpError::ConnectionError(e))
                        }
                    }
                    Ok(size)
                }
                Err(e) => Err(e)
            };
            let _ = ftp_stream.quit();
            ret
        }
        Err(e) => Err(e)
    }
}

fn sync() {
    let mut core = Core::new().unwrap();
    let https_client = hyper::Client::configure()
                        .connector(hyper_tls::HttpsConnector::new(4, &core.handle()).unwrap())
                        .build(&core.handle());
    

    let urls = [
        // ("delegated-arin-latest",             "https://ftp.arin.net/pub/stats/arin/delegated-arin-latest"),
        ("delegated-arin-extended-latest",    "https://ftp.arin.net/pub/stats/arin/delegated-arin-extended-latest"),
        ("delegated-ripencc-latest",          "https://ftp.ripe.net/pub/stats/ripencc/delegated-ripencc-latest"),
        ("delegated-ripencc-extended-latest", "https://ftp.ripe.net/pub/stats/ripencc/delegated-ripencc-extended-latest"),
        ("delegated-apnic-latest",            "ftp://ftp.apnic.net/public/stats/apnic/delegated-apnic-latest"),
        // ("delegated-apnic-extended-latest",   "ftp://ftp.apnic.net/public/stats/apnic/delegated-apnic-extended-latest"),
        ("delegated-lacnic-latest",           "http://ftp.lacnic.net/pub/stats/lacnic/delegated-lacnic-latest"),
        ("delegated-lacnic-extended-latest",  "http://ftp.lacnic.net/pub/stats/lacnic/delegated-lacnic-extended-latest"),
        ("delegated-afrinic-latest",          "https://ftp.afrinic.net/pub/stats/afrinic/delegated-afrinic-latest"),
        ("delegated-afrinic-extended-latest", "https://ftp.afrinic.net/pub/stats/afrinic/delegated-afrinic-extended-latest"),
    ];
    
    let (tx, rx) = channel::<(usize, bool)>();

    let mut task_idx = 0;
    for &(filename, url) in urls.iter() {
        let uri: hyper::Uri = url.parse().unwrap();

        if uri.scheme() == Some("http") || uri.scheme() == Some("https") {
            let md5_file_url = format!("{}.md5", url);
            let local_md5_file_path = Path::new(DATA_DIR).join(format!("{}.md5", filename));

            let job = https_client.get(md5_file_url.parse().unwrap())
                    .and_then(move |res| {
                        use futures::Stream;

                        let status_code = res.status().as_u16();
                        res.body()
                            .concat2()
                            .and_then(move |v|{
                                futures::future::ok((status_code, String::from_utf8(v.to_vec()).unwrap()))
                            })
                    })
                    .map(move |(status_code, body)|{
                        if status_code == 200 {
                            if local_md5_file_path.exists() {
                                let mut local_file = File::open(&local_md5_file_path).unwrap();
                                let mut local_md5_file_content = String::new();
                                local_file.read_to_string(&mut local_md5_file_content).unwrap();
                                drop(local_file);

                                if body != local_md5_file_content {
                                    // Diff
                                    let mut file = OpenOptions::new().create(true).write(true)
                                        .open(&local_md5_file_path).unwrap();
                                    file.write(&body.as_bytes()).unwrap();
                                    Ok(false)
                                } else {
                                    Ok(true)
                                }
                            } else {
                                let mut file = OpenOptions::new().create(true).write(true).append(false)
                                        .open(&local_md5_file_path).unwrap();
                                file.write(&body.as_bytes()).unwrap();
                                Ok(false)
                            }
                        } else {
                            Err(hyper::error::Error::Status)
                        }
                    });
            
            match core.run(job).unwrap() {
                Err(_) => tx.send((task_idx, false)).unwrap(),
                Ok(is_updated) => {
                    if !is_updated {
                        println!("Update {:?}", url);
                        let job2 = https_client.get(uri)
                            .and_then(move |res| {
                                let status_code = res.status().as_u16();
                                let data_dir = get_data_dir();
                                let mut file = OpenOptions::new().create(true).write(true)
                                            .open(data_dir.join(filename)).unwrap();

                                use futures::Stream;
                                res.body().for_each(move |chunk| {
                                    if status_code == 200 {
                                        file.write(&chunk).unwrap();
                                        future::ok(())
                                    } else {
                                        future::err(hyper::error::Error::Status)
                                    }
                                }).and_then(move |_|{
                                    if status_code == 200 {
                                        futures::future::ok(())
                                    } else {
                                        futures::future::err(hyper::error::Error::Status)
                                    }
                                })
                            });
                        match core.run(job2) {
                            Ok(_) => tx.send((task_idx, true)).unwrap(),
                            Err(_) => tx.send((task_idx, false)).unwrap()
                        };
                    } else {
                        tx.send((task_idx, true)).unwrap();
                    }
                }
            }
        } else if uri.scheme() == Some("ftp") {
            let sa = format!("{}:{}", uri.host().unwrap(), uri.port().unwrap_or(21) );
            let md5_file_url = format!("{}.md5", uri.path());
            let local_md5_file_path = Path::new(DATA_DIR).join(format!("{}.md5", filename));

            let mut is_updated = false;

            let mut md5_file_buffer = Cursor::new(vec![0u8; 4096]);
            md5_file_buffer.set_position(0);
            match ftp_get(&sa, &md5_file_url, &mut md5_file_buffer) {
                Ok(amt) => {
                    let _data = md5_file_buffer.into_inner();
                    let data = &_data[..amt];
                    let md5_file_content = String::from_utf8(data.to_vec()).unwrap();
                    
                    if local_md5_file_path.exists() {
                        let mut local_md5_file = File::open(&local_md5_file_path).unwrap();
                        let mut local_md5_file_content = String::new();
                        local_md5_file.read_to_string(&mut local_md5_file_content).unwrap();
                        if md5_file_content != local_md5_file_content {
                            // Diff
                            let mut file = OpenOptions::new().create(true).write(true)
                                    .open(&local_md5_file_path)
                                    .unwrap();
                            file.write(&md5_file_content.as_bytes()).unwrap();
                        } else {
                            is_updated = true;
                            tx.send((task_idx, true)).unwrap();
                        }
                    } else {
                        let mut file = OpenOptions::new().write(true).create(true)
                                .open(&local_md5_file_path)
                                .unwrap();
                        file.write_all(&md5_file_content.as_bytes()).unwrap();
                    }
                }
                Err(e) => {
                    println!("[ERROR] failure to download MD5 file({}). {:?}", url, e);
                    tx.send((task_idx, true)).unwrap();
                }
            }

            if is_updated == false {
                // update
                let mut file = OpenOptions::new().create(true).write(true)
                                    .open(get_data_dir().join(filename))
                                    .unwrap();
                match ftp_get(&sa, uri.path(), &mut file) {
                    Ok(_) => {
                        tx.send((task_idx, true)).unwrap();
                    }
                    Err(_) => {
                        tx.send((task_idx, false)).unwrap();
                    }
                }
            }
        } else {
            tx.send((task_idx, false)).unwrap();
            println!("ERROR: URL scheme not support: {:?}", url);
        }
        task_idx += 1;
    }

    
    {
        // ("delegated-iana-latest",             "ftp://ftp.apnic.net/public/stats/iana/delegated-iana-latest"),
        // ("delegated-iana-extended-latest",    "ftp://ftp.apnic.net/public/stats/iana/delegated-iana-extended-latest"),
        let filename = "delegated-iana-latest";
        let fileurl = "ftp://ftp.apnic.net/public/stats/iana/delegated-iana-latest";
        let mut file = OpenOptions::new().create(true).write(true)
                            .open(get_data_dir().join(filename))
                            .unwrap();
        let uri: hyper::Uri = fileurl.parse().unwrap();
        let sa = format!("{}:{}", uri.host().unwrap(), uri.port().unwrap_or(21));

        match ftp_get(&sa, uri.path(), &mut file) {
            Ok(_) => {
                println!("GET true {}", fileurl);
            }
            Err(_) => {
                println!("GET false {}", fileurl);
            }
        }
    }
    

    let total_tasks = urls.len() - 1;
    let mut tasks = 0;
    let timeout = Duration::new(3, 0);
    loop {
        if tasks == total_tasks {
            println!("\n[INFO] DONE.");
            break;
        }

        core.turn(Some(timeout));

        match rx.try_recv() {
            Ok((idx, state)) => {
                tasks += 1;
                println!("GET {} {}", state, urls[idx].1);
            }
            Err(_) => { }
        }
    }
}

fn parse() {
    let filenames = [
        // "delegated-arin-latest",
        "delegated-arin-extended-latest",
        "delegated-ripencc-latest",
        "delegated-ripencc-extended-latest",
        "delegated-apnic-latest",
        // "delegated-apnic-extended-latest",
        "delegated-lacnic-latest",
        "delegated-lacnic-extended-latest",
        "delegated-afrinic-latest",
        "delegated-afrinic-extended-latest",

        "delegated-iana-latest",
        // "delegated-iana-extended-latest"
    ];

    let mut paths: Vec<PathBuf> = vec![];

    for filename in filenames.iter() {
        let path = Path::new(DATA_DIR).join(filename);
        if path.exists() && path.is_file() {
            paths.push(path.to_path_buf());
        }
    }

    let parse_ip_record_line = |line: &str| -> Option<Record> {
        let fields: Vec<&str> = line.split("|").collect();
        if fields.len() >= 7 {
            let src_registry = Registry::from_str(fields[0]).unwrap();
            let country_code = Country::from_str( if fields[1].trim() == "" { "ZZ" } else { fields[1] } ).unwrap();
            let type_  = fields[2];

            if type_ == "ipv4" {
                let start: Ipv4Addr  = fields[3].parse().unwrap();
                let start_ip = Ipv4Address::from_bytes(&start.octets());
                let num: usize = fields[4].parse().unwrap();

                let status_ = fields[6];
                let (status, dst_registry) = if src_registry == Registry::Iana {
                    (Status::Allocated, Some(Registry::from_str(status_).unwrap()))
                } else {
                    (Status::from_str(status_).unwrap(), None)
                };
                return Some(Record::Ipv4(Ipv4Record {
                    src_registry: src_registry,
                    country: country_code,
                    start: start_ip,
                    num: num,
                    status: status,
                    dst_registry: dst_registry
                }));
            } else if type_ == "ipv6" {
                let start: Ipv6Addr  = fields[3].parse().unwrap();
                let start_ip = Ipv6Address::from_bytes(&start.octets());
                let prefix: u8 = fields[4].parse().unwrap();

                let status_ = fields[6];
                let (status, dst_registry) = if src_registry == Registry::Iana {
                    (Status::Allocated, Some(Registry::from_str(status_).unwrap()))
                } else {
                    (Status::from_str(status_).unwrap(), None)
                };
                return Some(Record::Ipv6(Ipv6Record {
                    src_registry: src_registry,
                    country: country_code,
                    start: start_ip,
                    prefix: prefix,
                    status: status,
                    dst_registry: dst_registry
                }));
            } else {
                return None;
            }
        } else {
            return None;
        }
    };

    let mut records = HashSet::new();

    let output_file_path = get_data_dir().join("all");

    fs::remove_file(&output_file_path).unwrap();
    let mut output_file = OpenOptions::new().create(true).write(true).append(true)
                            .open(&output_file_path)
                            .unwrap();
    for filepath in paths {
        let mut file = File::open(&filepath).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        let mut line_idx = 0;
        for line in content.lines() {
            if line.starts_with("#") {
                continue;
            }

            if line_idx == 0 || line.ends_with("summary") {
                line_idx += 1;
                continue;
            }
            
            match parse_ip_record_line(line) {
                Some(record) => {
                    if records.insert(record) {
                        output_file.write_all( format!("{}\n", record).as_bytes() ).unwrap();
                    }
                }
                None => {  }
            };
            line_idx += 1;
        }
        
    }
    println!("[INFO] Total records: {:?}", records.len());
    println!("[INFO] DONE.");
}


/**
    $ iana-tool sync
    $ iana-tool report --ip-version 0 | 4 | 6 --detail
    $ iana-tool parse \
                --registry ALL | ARIN | RIPENCC | APNIC | LACNIC | AFRINIC | IETF \
                --country ALL | US | CN | JP | HK | UK | ... \
                --format rir | cidr | range \
                --output output.txt \
                --ip-version 0 | 4 | 6
    
    $ iana-tool route --country CN --gateway 39.35.160.11 --exculde "22.22.22.0/24; 33.22.44.2/32"
**/

fn boot () {
    #[allow(unused_imports)]
    use clap::{App, SubCommand, Arg};

    let app = App::new("IANA IP DB TOOLS")
        .version("0.1")
        .author("Luozijun <gnulinux@126.com>")
        .about("IP DB tools")
        .subcommand(
            SubCommand::with_name("sync")
                .about("Sync ip db from IANA")
        )
        .subcommand(
            SubCommand::with_name("parse")
                .about("Parse IANA RIR db file")
        );

    let matches = app.get_matches();

    if let Some(_sub_m) = matches.subcommand_matches("sync") {
        sync();
    } else if let Some(_sub_m) = matches.subcommand_matches("parse") {
        parse();
    } else {
        println!("{}", &matches.usage());
    }
}


fn main() {
    boot();
}


