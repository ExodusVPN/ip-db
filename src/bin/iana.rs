#[allow(unused_imports)]
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate clap;
extern crate iana;

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

use clap::{App, SubCommand, Arg};

use std::env;
use std::io::Write;
use std::fs::{self, OpenOptions};
use std::path::Path;


fn main () {
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
        )
        .arg(
            Arg::with_name("data-path")
                .long("data-path")
                .required(false)
                .default_value("data")
                .help("Specify the default data path")
        );

    let matches = app.get_matches();


    env::set_var("RUST_LOG", "iana=debug");
    env_logger::init();


    let data_path = Path::new(matches.value_of("data-path").unwrap().to_lowercase().as_str()).to_path_buf();
    if !data_path.exists() {
        fs::create_dir(&data_path).unwrap();
    }

    if let Some(_sub_m) = matches.subcommand_matches("sync") {
        iana::sync(&data_path);
    } else if let Some(_sub_m) = matches.subcommand_matches("parse") {
        let records = iana::parse(&data_path);
        let mut output_file = OpenOptions::new().create(true).write(true).append(true)
                            .open("all")
                            .unwrap();
        for record in records.iter() {
            output_file.write(format!("{}\n", record).as_bytes()).unwrap();
        }
    } else {
        println!("{}", &matches.usage());
    }
}
