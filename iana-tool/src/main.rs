#[macro_use]
extern crate log;
extern crate env_logger;

extern crate smoltcp;
extern crate clap;


extern crate ip_db;

mod sync;
mod rir;
mod ipv4_range;



use smoltcp::wire::{ Ipv4Address, Ipv6Address };


use std::str::FromStr;
use std::io::{Write, Read};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::fs::{self, File, OpenOptions};
use std::collections::HashSet;


use ip_db::{Registry, Country, Status};
use rir::{Record, Ipv4Record, Ipv6Record};


fn parse(data_path: PathBuf, format: String) {
    let filenames = [
        // "delegated-arin-latest",
        "delegated-arin-extended-latest",
        "delegated-ripencc-latest",
        "delegated-ripencc-extended-latest",
        "delegated-apnic-latest",
        "delegated-apnic-extended-latest",
        "delegated-lacnic-latest",
        "delegated-lacnic-extended-latest",
        "delegated-afrinic-latest",
        "delegated-afrinic-extended-latest",

        "delegated-iana-latest",
        // "delegated-iana-extended-latest"
    ];

    let mut paths: Vec<PathBuf> = vec![];

    for filename in filenames.iter() {
        let path = data_path.join(filename);
        if path.exists() && path.is_file() {
            paths.push(path.to_path_buf());
        }
    }

    let parse_ip_record_line = |line: &str| -> Option<Record> {
        let fields: Vec<&str> = line.split("|").collect();
        if fields.len() >= 7 {
            let src_registry = Registry::from_str(fields[0]).unwrap();
            let cc = if fields[1].trim() == "" { "ZZ" } else { fields[1] };
            let country_code = Country::from_str(cc).unwrap();
            let type_  = fields[2];

            if type_ == "ipv4" {
                let start: Ipv4Addr  = fields[3].parse().unwrap();
                let start_ip = Ipv4Address(start.octets());
                let num: u32 = fields[4].parse().unwrap();

                assert!(num > 0);

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
                let start_ip = Ipv6Address(start.octets());
                let prefix: u8 = fields[4].parse().unwrap();

                assert!(prefix <= 128);

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

    let output_file_path = data_path.join("all");
    let _ = fs::remove_file(&output_file_path);
    
    println!("Output file: {:?}", output_file_path);

    let mut output_file = OpenOptions::new().create(true).write(true).append(true)
                            .open(&output_file_path)
                            .unwrap();
    for filepath in paths {
        println!("parse {:?} ...", filepath);

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
                        let oline = if format == "rir" {
                            record.to_rir()
                        } else if format == "cidr" {
                            record.to_cidr()
                        } else {
                            unreachable!();
                        };
                        output_file.write_all( format!("{}\n", oline ).as_bytes() ).unwrap();
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
                .arg(
                    Arg::with_name("format")
                        .long("format")
                        .required(false)
                        .default_value("rir")
                        .help("Specify the output format(RIR/CIDR)")
                )
        )
        .arg(
            Arg::with_name("data-path")
                .long("data-path")
                .required(false)
                .default_value("data")
                .help("Specify the default data path")
        );

    let matches = app.get_matches();

    let path_ = matches.value_of("data-path").unwrap().to_lowercase();
    let data_path = Path::new(path_.as_str());
    if data_path.exists() == false {
        fs::create_dir(data_path).unwrap();
    }
    
    if let Some(_sub_m) = matches.subcommand_matches("sync") {
        sync::sync(data_path.to_path_buf());
    } else if let Some(_sub_m) = matches.subcommand_matches("parse") {
        let format = _sub_m.value_of("format").unwrap().to_lowercase();
        if format != "rir" && format != "cidr" {
            println!("{}", &_sub_m.usage());
        } else {
            parse(data_path.to_path_buf(), format);
        }
    } else {
        println!("{}", &matches.usage());
    }
}


fn main() {
    use std::env;
    
    env::set_var("RUST_LOG", "iana_tool=debug");
    env_logger::init();

    boot();
}


