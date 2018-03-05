#[allow(unused_imports)]
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate clap;
extern crate iana;

use clap::{App, SubCommand, Arg};

use std::env;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use std::net::{IpAddr};
use std::fs::{self, OpenOptions};



fn main () {
    env::set_var("RUST_LOG", "iana=debug");
    env_logger::init();

    let app = App::new("IANA IP DB TOOLS")
        .version("0.1")
        .author("Luozijun <luozijun.assistant@gmail.com>")
        .about("IP DB tools")
        .subcommand(
            SubCommand::with_name("sync")
                .about("Sync ip db from IANA")
                .arg(
                    Arg::with_name("data-path")
                        .long("data-path")
                        .required(false)
                        .default_value("data")
                        .help("Specify the default data path")
                )
        )
        .subcommand(
            SubCommand::with_name("parse")
                .about("Parse IANA RIR db file")
                .arg(
                    Arg::with_name("data-path")
                        .long("data-path")
                        .required(false)
                        .default_value("data")
                        .help("Specify the default data path")
                )
        )
        .subcommand(
            SubCommand::with_name("lookup")
                .about("Query an IP location")
                .arg(
                    Arg::with_name("ipaddr")
                        .required(true)
                        .help("Specify the IP Address")
                )
        );
        

    let matches = app.get_matches();

    if let Some(_sub_m) = matches.subcommand_matches("sync") {
        let data_path = Path::new(_sub_m.value_of("data-path").unwrap().to_lowercase().as_str()).to_path_buf();
        if !data_path.exists() {
            fs::create_dir(&data_path).unwrap();
        }

        iana::sync(&data_path);
    } else if let Some(_sub_m) = matches.subcommand_matches("parse") {
        let data_path = Path::new(_sub_m.value_of("data-path").unwrap().to_lowercase().as_str()).to_path_buf();
        if !data_path.exists() {
            fs::create_dir(&data_path).unwrap();
        }

        let record_sets = iana::parse(&data_path);

        let mut v4_records: Vec<&iana::Record> = record_sets.iter().filter(|record| {
            record.is_ipv4() && record.src_registry() != iana::Registry::Iana
        } ).collect();
        let mut v6_records: Vec<&iana::Record> = record_sets.iter().filter(|record| {
            record.is_ipv6() && record.src_registry() != iana::Registry::Iana
        } ).collect();

        let mut iana_v4_records: Vec<&iana::Record> = record_sets.iter().filter(|record| {
            record.is_ipv4() && record.dst_registry().is_some() && record.src_registry() == iana::Registry::Iana
        } ).collect();
        let mut iana_v6_records: Vec<&iana::Record> = record_sets.iter().filter(|record| {
            record.is_ipv6() && record.dst_registry().is_some() && record.src_registry() == iana::Registry::Iana
        } ).collect();

        v4_records.sort_unstable();
        v6_records.sort_unstable();
        iana_v4_records.sort_unstable();
        iana_v6_records.sort_unstable();

        let v4_output_filepath = data_path.join("v4_records");
        let v6_output_filepath = data_path.join("v6_records");
        let iana_v4_output_filepath = data_path.join("iana_v4_records");
        let iana_v6_output_filepath = data_path.join("iana_v6_records");

        let v4_db_filepath = "src/v4_db.rs";
        let v6_db_filepath = "src/v6_db.rs";

        let _ = fs::remove_file(&v4_output_filepath);
        let _ = fs::remove_file(&v6_output_filepath);
        let _ = fs::remove_file(&iana_v4_output_filepath);
        let _ = fs::remove_file(&iana_v6_output_filepath);

        let _ = fs::remove_file(&v4_db_filepath);
        let _ = fs::remove_file(&v6_db_filepath);

        let mut v4_output_file = OpenOptions::new().create(true).write(true).append(true)
                            .open(&v4_output_filepath)
                            .unwrap();
        let mut v6_output_file = OpenOptions::new().create(true).write(true).append(true)
                            .open(&v6_output_filepath)
                            .unwrap();
        let mut iana_v4_output_file = OpenOptions::new().create(true).write(true).append(true)
                            .open(&iana_v4_output_filepath)
                            .unwrap();
        let mut iana_v6_output_file = OpenOptions::new().create(true).write(true).append(true)
                            .open(&iana_v6_output_filepath)
                            .unwrap();
        
        let mut v4_db_file = OpenOptions::new().create(true).write(true).append(true)
                            .open(&v4_db_filepath)
                            .unwrap();
        let mut v6_db_file = OpenOptions::new().create(true).write(true).append(true)
                            .open(&v6_db_filepath)
                            .unwrap();

        for record in v4_records.iter() {
            v4_output_file.write(format!("{}\n", record).as_bytes()).unwrap();
        }
        for record in v6_records.iter() {
            v6_output_file.write(format!("{}\n", record).as_bytes()).unwrap();
        }

        for record in iana_v4_records.iter() {
            iana_v4_output_file.write(format!("{}\n", record).as_bytes()).unwrap();
        }
        for record in iana_v6_records.iter() {
            iana_v6_output_file.write(format!("{}\n", record).as_bytes()).unwrap();
        }

        let v4_db = v4_records.iter().map(|record| record.codegen()).collect::<Vec<String>>();
        let v6_db = v6_records.iter().map(|record| record.codegen()).collect::<Vec<String>>();

        v4_db_file.write(format!("pub const IPV4_RECORDS: [(u32, u32, u8); {}] = [\n{}\n];", v4_db.len(), v4_db.join(",\n")).as_bytes()).unwrap();
        v6_db_file.write(format!("pub const IPV6_RECORDS: [([u8; 16], [u8; 16], u8); {}] = [\n{}\n];", v6_db.len(), v6_db.join(",\n")).as_bytes()).unwrap();

    } else if let Some(_sub_m) = matches.subcommand_matches("lookup") {
        let ipaddr: IpAddr = _sub_m.value_of("ipaddr").unwrap().to_lowercase().parse().unwrap();
        
        println!("Lookup: {:?}", &ipaddr);

        let now = Instant::now();
        let res = iana::lookup(&ipaddr);
        let duration = now.elapsed();

        println!("Item: {:?}", res);
        println!("Duration: {:?}", duration);

    } else {
        println!("{}", &matches.usage());
    }
}
