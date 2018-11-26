
mod country;
mod registry;
mod status;
mod error;

#[cfg( all(not(feature = "sync"), not(feature = "parse")) )]
mod v4_db;
#[cfg( all(not(feature = "sync"), not(feature = "parse")) )]
mod v6_db;

#[cfg(any(feature = "sync", feature = "parse"))]
mod v4_db {
    pub static IPV4_RECORDS: [(u32, u32, u8); 0] = [];
}
#[cfg(any(feature = "sync", feature = "parse"))]
mod v6_db {
    pub static IPV6_RECORDS: [(u128, u128, u8); 0] = [];
}


pub use country::Country;
pub use registry::Registry;
pub use status::Status;
pub use error::Error;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::cmp::Ordering;


// Files not exists
// ("delegated-arin-latest",             "https://ftp.arin.net/pub/stats/arin/delegated-arin-latest"),
// ("delegated-iana-extended-latest",    "ftp://ftp.apnic.net/public/stats/iana/delegated-iana-extended-latest"),
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
            let v6_number = u128::from(v6_addr);
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
                    Some( (IpAddr::from(Ipv6Addr::from(first)),
                           IpAddr::from(Ipv6Addr::from(last)),
                           Country::from_index(cc).unwrap() ))
                }
                Err(_) => None
            }
        }
    }
}


#[test]
fn test_lookup_ipv4() {
    assert_eq!(lookup(&IpAddr::from(Ipv4Addr::new(8, 8, 8, 8))).is_some(), true);
}

#[test]
fn test_lookup_ipv6() {
    assert_eq!(lookup(&"2001:218::".parse().unwrap()).is_some(), true);
}
