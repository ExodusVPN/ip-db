#![feature(i128_type)]

#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(feature = "sync")] {
        extern crate futures;
        extern crate tokio_core;
        extern crate hyper;
        extern crate hyper_tls;
        extern crate smoltcp;
    } else if #[cfg(feature = "parse")] {
        extern crate smoltcp;
    }
}


mod country;
mod registry;
mod status;
mod error;

#[cfg(feature = "sync")]
mod sync;
#[cfg(feature = "parse")]
mod parse;


pub use country::Country;
pub use registry::Registry;
pub use status::Status;
#[cfg(feature = "sync")]
pub use sync::sync;
#[cfg(feature = "parse")]
pub use parse::{parse, Record, IpBlock, Ipv4Range};


// Files not exists
// ("delegated-arin-latest",             "https://ftp.arin.net/pub/stats/arin/delegated-arin-latest"),
// ("delegated-iana-extended-latest",    "ftp://ftp.apnic.net/public/stats/iana/delegated-iana-extended-latest"),
#[cfg(any(feature = "sync", feature = "parse"))]
pub const IANA_RIR_FILES: [(&str, &str); 10] = [    
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
];

