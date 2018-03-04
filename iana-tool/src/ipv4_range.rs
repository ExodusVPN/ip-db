#![allow(dead_code)]

use smoltcp::wire::{Ipv4Address, Ipv4Cidr};

use std::cmp::min;
use std::net::Ipv4Addr;
use std::iter::Iterator;


#[derive(Debug)]
pub struct Ipv4Range {
    start_ip: Ipv4Address,
    end_ip  : Ipv4Address,
}

impl Ipv4Range {
    pub fn new(start_ip: Ipv4Address, end_ip: Ipv4Address) -> Self {
        Ipv4Range { start_ip, end_ip }
    }
    
    pub fn first(&self) -> Ipv4Address {
        self.start_ip
    }

    pub fn last(&self) -> Ipv4Address {
        self.end_ip
    }

    pub fn total(&self) -> u32 {
        u32::from(Ipv4Addr::from(self.end_ip.0)) - u32::from(Ipv4Addr::from(self.start_ip.0)) + 1
    }

    pub fn addrs(&self) -> Ipv4AddrsIter {
        Ipv4AddrsIter {
            offset: u32::from(Ipv4Addr::from(self.start_ip.0)) as u64,
            end   : u32::from(Ipv4Addr::from(self.end_ip.0)) as u64,
        }
    }

    pub fn cidrs(&self) -> Ipv4CidrIter {
        Ipv4CidrIter {
            start: u32::from(Ipv4Addr::from(self.start_ip.0)) as u64,
            end  : u32::from(Ipv4Addr::from(self.end_ip.0)) as u64,
        }
    }
}

pub struct Ipv4AddrsIter {
    offset: u64,
    end: u64,
}

impl Iterator for Ipv4AddrsIter {
    type Item = Ipv4Address;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end >= self.offset {
            let ip = Ipv4Addr::from(self.offset as u32);
            self.offset += 1;
            Some(Ipv4Address(ip.octets()))
        } else {
            None
        }
    }
}

pub struct Ipv4CidrIter {
    start: u64,
    end  : u64,
}

impl Iterator for Ipv4CidrIter {
    type Item = Ipv4Cidr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end > self.start {
            let mut shift = min(32, self.start.trailing_zeros());
            let num: u64;

            loop {
                let n = 2u64.pow(shift);
                if self.start + n > self.end + 1 {
                    if shift == 0 {
                        panic!("oops ...")
                    }
                    shift -= 1;
                } else {
                    num = n;
                    break;
                }
            }
            let prefix_len = 32 - shift;
            let ip = Ipv4Addr::from(self.start as u32);
            let cidr = Ipv4Cidr::new(Ipv4Address(ip.octets()), prefix_len as u8);
            self.start += num;
            Some(cidr)
        } else if self.end == self.start {
            let ip = Ipv4Addr::from(self.end as u32);
            let cidr = Ipv4Cidr::new(Ipv4Address(ip.octets()), 32);
            self.start += 1;
            Some(cidr)
        } else {
            None
        }
    }
}

#[test]
fn test_ipv4range_to_ipv4cidrs() {
    {
        // 0.0.0.0 - 0.255.255.255 | 16777216
        // let end_ip = Ipv4Address( Ipv4Addr::from( u32::from(Ipv4Addr::new(0, 0, 0, 0)) + (16777216 - 1) ).octets() );
        assert_eq!(Ipv4Range::new( Ipv4Address::new(0, 0, 0, 0), Ipv4Address::new(0, 255, 255, 255) ).total(),
               16777216);
        assert_eq!(Ipv4Range::new( Ipv4Address::new(0, 0, 0, 0), Ipv4Address::new(0, 255, 255, 255) ).addrs().count(),
               16777216);
    }

    {
        // 255.0.0.0 - 255.255.255.255 | 16777216
        assert_eq!(Ipv4Range::new( Ipv4Address::new(255, 0, 0, 0), Ipv4Address::new(255, 255, 255, 255) ).total(),
               16777216);
        assert_eq!(Ipv4Range::new( Ipv4Address::new(255, 0, 0, 0), Ipv4Address::new(255, 255, 255, 255) ).addrs().count(),
               16777216);
    }
    
    // 185.30.232.0 - 185.30.232.63  64
    assert_eq!(Ipv4Range::new( Ipv4Address::new(185, 30, 232, 0), Ipv4Address::new(185, 30, 232, 63) ).total(),
               64);
    assert_eq!(Ipv4Range::new( Ipv4Address::new(185, 30, 232, 0), Ipv4Address::new(185, 30, 232, 63) ).addrs().count(),
              64);

    // 23.18.1.0 - 23.18.23.255   5888
    assert_eq!(Ipv4Range::new( Ipv4Address::new(23, 18, 1, 0), Ipv4Address::new(23, 18, 23, 255) ).total(),
              5888);
    assert_eq!(Ipv4Range::new( Ipv4Address::new(23, 18, 1, 0), Ipv4Address::new(23, 18, 23, 255) ).addrs().count(),
              5888);
    // 23.18.1.0/24
    // 23.18.2.0/23
    // 23.18.4.0/22
    // 23.18.8.0/21
    // 23.18.16.0/22
    // 23.18.20.0/23
    // 23.18.22.0/24
    // 23.18.23.0/25
    // 23.18.23.128/26
    // 23.18.23.192/27
    // 23.18.23.224/28
    // 23.18.23.240/29
    // 23.18.23.248/30
    // 23.18.23.252/31
    // 23.18.23.254/32
    // 23.18.23.255/32
    assert_eq!(Ipv4Range::new( Ipv4Address::new(23, 18, 1, 0), Ipv4Address::new(23, 18, 23, 255) ).cidrs().collect::<Vec<Ipv4Cidr>>(),
              vec![
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 1, 0), 24),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 2, 0), 23),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 4, 0), 22),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 8, 0), 21),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 16, 0), 22),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 20, 0), 23),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 22, 0), 24),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 23, 0), 25),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 23, 128), 26),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 23, 192), 27),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 23, 224), 28),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 23, 240), 29),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 23, 248), 30),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 23, 252), 31),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 23, 254), 32),
                Ipv4Cidr::new(Ipv4Address::new(23, 18, 23, 255), 32),
              ]);
}
