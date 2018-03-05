#![feature(i128_type, test)]
extern crate test;
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
mod v4_db;
mod v6_db;

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

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::cmp::Ordering;


// Files not exists
// ("delegated-arin-latest",             "https://ftp.arin.net/pub/stats/arin/delegated-arin-latest"),
// ("delegated-iana-extended-latest",    "ftp://ftp.apnic.net/public/stats/iana/delegated-iana-extended-latest"),
#[cfg(any(feature = "sync", feature = "parse"))]
pub static IANA_RIR_FILES: [(&str, &str); 10] = [
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


pub fn lookup(ip: &IpAddr) -> Option<(IpAddr, IpAddr, Country)> {
    match ip {
        &IpAddr::V4(v4_addr) => {
            let v4_number = u32::from(v4_addr);
            let ret = v4_db::IPV4_RECORDS.binary_search_by(|&(first, last, _cc)| {
                if v4_number >= last {
                    Ordering::Less
                } else if v4_number >= first && v4_number <= last {
                    Ordering::Equal
                } else if v4_number < first {
                    Ordering::Greater
                } else {
                    unreachable!()
                }
            });
            match ret {
                Ok(pos) => {
                    let (first, last, cc) = v4_db::IPV4_RECORDS[pos];
                    Some( (IpAddr::from(Ipv4Addr::from(first)),
                           IpAddr::from(Ipv4Addr::from(last)),
                           Country::from_index(cc).unwrap() ))
                }
                Err(_) => None
            }
        }
        &IpAddr::V6(v6_addr) => {
            let v6_number = v6_addr.octets();
            let ret = v6_db::IPV6_RECORDS.binary_search_by(|&(first, last, _cc)| {
                if v6_number >= last {
                    Ordering::Less
                } else if v6_number >= first && v6_number <= last {
                    Ordering::Equal
                } else if v6_number < first {
                    Ordering::Greater
                } else {
                    unreachable!()
                }
            });

            match ret {
                Ok(pos) => {
                    let (first, last, cc) = v6_db::IPV6_RECORDS[pos];
                    Some( (IpAddr::from(first),
                           IpAddr::from(last),
                           Country::from_index(cc).unwrap() ))
                }
                Err(_) => None
            }
        }
    }
}

#[bench]
fn bench_lookup_ipv4(b: &mut test::Bencher) {
    extern crate rand;

    let ip = IpAddr::from(Ipv4Addr::from([
        rand::random::<u8>(), rand::random::<u8>(),
        rand::random::<u8>(), rand::random::<u8>(),
    ]));

    b.iter(|| {
        let _ = lookup(&ip);
    })
}

#[bench]
fn bench_lookup_ipv6(b: &mut test::Bencher) {
    extern crate rand;

    let ip = IpAddr::from(Ipv6Addr::from([
        rand::random::<u16>(), rand::random::<u16>(),
        rand::random::<u16>(), rand::random::<u16>(),
        rand::random::<u16>(), rand::random::<u16>(),
        rand::random::<u16>(), rand::random::<u16>(),
    ]));
    
    b.iter(|| {
        let _ = lookup(&ip);
    })
}
