
#[allow(unused_imports)]
use smoltcp::wire::{
    IpAddress, IpCidr, 
    Ipv4Address, Ipv4Cidr, 
    Ipv6Address, Ipv6Cidr
};

use ip_db::{Registry, Country, Status};

use std::fmt;
use std::cmp::Ordering;


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd)]
pub struct Ipv4Record {
    pub src_registry: Registry,
    pub country: Country,
    pub start: Ipv4Address,
    pub num: usize,
    pub status: Status,
    pub dst_registry: Option<Registry>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd)]
pub struct Ipv6Record {
    pub src_registry: Registry,
    pub country: Country,
    pub start: Ipv6Address,
    pub prefix: u8,
    pub status: Status,
    pub dst_registry: Option<Registry>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Record {
    Ipv4(Ipv4Record),
    Ipv6(Ipv6Record)
}

impl Ipv4Record {
    /// RIR format
    pub fn to_rir(&self) -> String {
        format!("{} {} ipv4 {} {} {} {}",
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

    // CIDR format
    pub fn to_cidr(&self) -> String {
        let nums = self.num - 1;
        let bits_len = nums.count_ones() + nums.count_zeros() - nums.leading_zeros();
        assert!(bits_len <= 32);
        
        let prefix_len = 32 - bits_len;
        let v4_block = Ipv4Cidr::new(self.start, prefix_len as u8);

        format!("{} {} ipv4 {} {} {}",
            self.src_registry,
            self.country,
            v4_block,
            self.status,
            match self.dst_registry {
                Some(reg) => format!("{}", reg),
                None => "none".to_string()
            })
    }
}

impl Ord for Ipv4Record {
    fn cmp(&self, other: &Ipv4Record) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl Ipv6Record {
    /// RIR format
    pub fn to_rir(&self) -> String {
        format!("{} {} ipv6 {} {} {} {}",
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

    // CIDR format
    pub fn to_cidr(&self) -> String {
        let v6_block = Ipv6Cidr::new(self.start, self.prefix);
        format!("{} {} ipv6 {} {} {}",
            self.src_registry,
            self.country,
            v6_block,
            self.status,
            match self.dst_registry {
                Some(reg) => format!("{}", reg),
                None => "none".to_string()
            })
    }
}

impl Ord for Ipv6Record {
    fn cmp(&self, other: &Ipv6Record) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl Record {
    /// RIR format
    pub fn to_rir(&self) -> String {
        match *self {
            Record::Ipv4(ipv4_record) => ipv4_record.to_rir(),
            Record::Ipv6(ipv6_record) => ipv6_record.to_rir(),
        }
    }

    // CIDR format
    pub fn to_cidr(&self) -> String {
        match *self {
            Record::Ipv4(ipv4_record) => ipv4_record.to_cidr(),
            Record::Ipv6(ipv6_record) => ipv6_record.to_cidr(),
        }
    }
}

impl fmt::Display for Ipv4Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_rir())
    }
}

impl fmt::Display for Ipv6Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_rir())
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