
use smoltcp::wire::{
    IpAddress,
    Ipv4Address, Ipv4Cidr,
    Ipv6Address, Ipv6Cidr,
};

use {Registry, Country, Status, IANA_RIR_FILES};

use std::fmt;
use std::cmp;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::collections::HashSet;



#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd)]
pub struct Ipv4Range {
    pub start_ip: Ipv4Address,
    pub end_ip  : Ipv4Address,
}

impl Ord for Ipv4Range {
    fn cmp(&self, other: &Ipv4Range) -> cmp::Ordering {
        self.start_ip.cmp(&other.start_ip)
    }
}

impl fmt::Display for Ipv4Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.start_ip, self.end_ip)
    }
}

impl Ipv4Range {
    pub fn new(start_ip: Ipv4Address, end_ip: Ipv4Address) -> Self {
        Ipv4Range { start_ip, end_ip }
    }
    
    pub fn with_nums(start_ip: Ipv4Address, nums: u32) -> Self {
        let end_ip_number = u32::from(Ipv4Addr::from(start_ip.0)) + (nums - 1);
        let end_ip = Ipv4Address( Ipv4Addr::from(end_ip_number).octets() );

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
            let mut shift = cmp::min(32, self.start.trailing_zeros());
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


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum IpBlock {
    Ipv4Range(Ipv4Range),
    Ipv4Cidr(Ipv4Cidr),
    Ipv6Cidr(Ipv6Cidr),
}

impl IpBlock {
    pub fn first(&self) -> IpAddress {
        match *self {
            IpBlock::Ipv4Range(v4_range) => IpAddress::Ipv4(v4_range.first()),
            IpBlock::Ipv4Cidr(v4_cidr) => {
                IpAddress::from(v4_cidr.network().address())
            },
            IpBlock::Ipv6Cidr(v6_cidr) => {
                IpAddress::from(v6_cidr.address())
            },
        }
    }

    pub fn last(&self) -> IpAddress {
        match *self {
            IpBlock::Ipv4Range(v4_range) => IpAddress::Ipv4(v4_range.last()),
            IpBlock::Ipv4Cidr(v4_cidr) => {
                
                let first_number = match self.first() {
                    IpAddress::Ipv4(v4_addr) => u32::from(Ipv4Addr::from(v4_addr)),
                    IpAddress::Ipv6(_) => unreachable!(),
                    _ => unreachable!()
                };

                let max = first_number + 2u32.pow(v4_cidr.prefix_len() as u32);
                IpAddress::from(Ipv4Address::from(Ipv4Addr::from(max)))
            },
            IpBlock::Ipv6Cidr(v6_cidr) => {
                let first_number = match self.first() {
                    IpAddress::Ipv4(_) => unreachable!(),
                    IpAddress::Ipv6(v6_addr) => u128::from(Ipv6Addr::from(v6_addr)),
                    _ => unreachable!()
                };

                let max = first_number + 2u128.pow(v6_cidr.prefix_len() as u32);
                IpAddress::from(Ipv6Address::from(Ipv6Addr::from(max)))
            },
        }
    }

    pub fn is_ipv4(&self) -> bool {
        match *self {
            IpBlock::Ipv4Range(_) | IpBlock::Ipv4Cidr(_) => true,
            IpBlock::Ipv6Cidr(_) => false,
        }
    }

    pub fn is_ipv6(&self) -> bool {
        match *self {
            IpBlock::Ipv4Range(_) | IpBlock::Ipv4Cidr(_) => false,
            IpBlock::Ipv6Cidr(_) => true,
        }
    }
}


impl fmt::Display for IpBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IpBlock::Ipv4Range(v4_range) => fmt::Display::fmt(&v4_range, f),
            IpBlock::Ipv4Cidr(v4_cidr) => fmt::Display::fmt(&v4_cidr, f),
            IpBlock::Ipv6Cidr(v6_cidr) => fmt::Display::fmt(&v6_cidr, f),
        }
    }
}


#[derive(Debug, Copy, Clone, Hash, Eq)]
pub struct Record {
    pub src_registry: Registry,
    pub country: Country,
    pub ip_block: IpBlock,
    pub status: Status,
    pub dst_registry: Option<Registry>,
}

impl Record {
    pub fn src_registry(&self) -> Registry {
        self.src_registry
    }

    pub fn country(&self) -> Country {
        self.country
    }

    pub fn type_(&self) -> String {
        if self.is_ipv4() {
            "ipv4".to_string()
        } else if self.is_ipv6() {
            "ipv6".to_string()
        } else {
            unreachable!()
        }
    }

    pub fn ip_version(&self) -> String {
        self.type_()
    }

    pub fn ip_block(&self) -> IpBlock {
        self.ip_block
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn dst_registry(&self) -> Option<Registry> {
        self.dst_registry
    }

    pub fn is_ipv4(&self) -> bool {
        self.ip_block.is_ipv4()
    }

    pub fn is_ipv6(&self) -> bool {
        self.ip_block.is_ipv6()
    }

    pub fn codegen(&self) -> String {
        let first_ip = self.ip_block.first();
        let last_ip = self.ip_block.last();
        
        let ip_to_number_string = |ipaddr| -> String {
            match ipaddr {
                IpAddress::Ipv4(v4_addr) => format!("{}", u32::from(Ipv4Addr::from(v4_addr))),
                IpAddress::Ipv6(v6_addr) => format!("[{}]", 
                    &v6_addr.0.iter().map(|d| format!("{}", d)).collect::<Vec<String>>().join(", ") ),
                _ => unreachable!()
            }
        };
        
        format!("({}, {}, {})",
                ip_to_number_string(first_ip),
                ip_to_number_string(last_ip),
                self.country.index())
    }
}

impl Ord for Record {
    fn cmp(&self, other: &Record) -> cmp::Ordering {
        self.ip_block.first().cmp(&other.ip_block.first())
    }
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Record) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.ip_block.first() == other.ip_block.first()
    }
}


impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {} {} {}",
            self.src_registry,
            self.country,
            self.type_(),
            match self.ip_block {
                IpBlock::Ipv4Range(v4_range) => format!("{} {}", v4_range.first(), v4_range.total()),
                IpBlock::Ipv4Cidr(v4_cidr) => format!("{} {}", v4_cidr.address(), v4_cidr.prefix_len()),
                IpBlock::Ipv6Cidr(v6_cidr) => format!("{} {}", v6_cidr.address(), v6_cidr.prefix_len()),
            },
            self.status,
            match self.dst_registry {
                Some(reg) => format!("{}", reg),
                None => "none".to_string()
            })
    }
}

impl FromStr for Record {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fields: Vec<&str> = s.split("|").collect();
        
        if fields.len() < 7 {
            debug!("Less than 7 fields length of this RIR Record");
            return Err(())
        }
        
        let src_registry = Registry::from_str(fields[0]).unwrap();
        let cc = if fields[1].trim() == "" { "ZZ" } else { fields[1] };
        let country_code = Country::from_str(cc).unwrap();
        let type_  = fields[2];

        match type_ {
            "ipv4" => {
                let start: Ipv4Addr = fields[3].parse().unwrap();
                let start_ip = Ipv4Address(start.octets());
                let nums: u32 = fields[4].parse().unwrap();

                assert!(nums > 0);

                let status_ = fields[6];
                let (status, dst_registry) = if src_registry == Registry::Iana {
                    (Status::Assigned, Some(Registry::from_str(status_).unwrap()))
                } else {
                    (Status::from_str(status_).unwrap(), None)
                };

                let ip_block = IpBlock::Ipv4Range(Ipv4Range::with_nums(start_ip, nums));

                let record = Record {
                    src_registry: src_registry,
                    country: country_code,
                    ip_block: ip_block,
                    status: status,
                    dst_registry: dst_registry
                };

                Ok(record)
            }
            "ipv6" => {
                let start: Ipv6Addr = fields[3].parse().unwrap();
                let start_ip = Ipv6Address(start.octets());
                let prefix_len: u8 = fields[4].parse().unwrap();

                assert!(prefix_len <= 128);

                let status_ = fields[6];
                let (status, dst_registry) = if src_registry == Registry::Iana {
                    (Status::Assigned, Some(Registry::from_str(status_).unwrap()))
                } else {
                    (Status::from_str(status_).unwrap(), None)
                };

                let ip_block = IpBlock::Ipv6Cidr(Ipv6Cidr::new(start_ip, prefix_len));

                let record = Record {
                    src_registry: src_registry,
                    country: country_code,
                    ip_block: ip_block,
                    status: status,
                    dst_registry: dst_registry
                };

                Ok(record)
            }
            _ => {
                trace!("Not an IPv4 or IPv6 Record Line.");
                Err(())
            }
        }
    }
}



pub fn parse(data_path: &PathBuf) -> HashSet<Record> {
    let mut records: HashSet<Record> = HashSet::new();

    if !data_path.exists() {
        debug!("Path {:?} not exists", data_path);
       return records;
    }

    let filepaths: Vec<PathBuf> = IANA_RIR_FILES.iter().map(|&(filename, _)| data_path.join(filename) ).collect();

    for filepath in filepaths {
        info!("Parse {:?} ...", filepath);
        if !filepath.exists() {
            error!("FilePath {:?} not exists.", &filepath);
            continue;
        }
        
        if !filepath.is_file() {
            error!("FilePath {:?} not a file.", &filepath);
            continue;
        }
        
        let file_content = {
            let mut file = File::open(&filepath).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            content
        };
        
        let mut line_idx = 0usize;

        for line in file_content.lines() {
            if line.starts_with("#") {
                trace!("Ignore comment line#{}", line_idx);
                continue;
            }

            if line_idx == 0 || line.ends_with("summary") {
                trace!("Ignore summary line#{}", line_idx);
                line_idx += 1;
                continue;
            }
            
            if let Ok(record) = Record::from_str(line) {
                records.insert(record);
            } else {
                trace!("Parse Line#{} failed.", line_idx);
            }

            line_idx += 1;
        }
    }

    records
}

