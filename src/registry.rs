
use std::str::FromStr;
use std::fmt;

use crate::error::Error;


/// Number Resources
/// 
/// `https://www.iana.org/numbers`
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd)]
pub enum Registry {
    /// Africa Region
    Afrinic = 0u8,
    /// Asia/Pacific Region
    Apnic,
    /// Canada, USA, and some Caribbean Islands
    Arin,
    /// Internet Assigned Numbers Authority(IANA)
    Iana,
    /// Internet Engineering Task Force(IETF), Special Registry
    Ietf,
    /// Latin America and some Caribbean Islands
    Lacnic,
    /// Europe, the Middle East, and Central Asia
    Ripencc,
}

impl Registry {
    pub fn from_index(index: u8) -> Result<Self, ()> {
        match index {
            0u8 => Ok(Registry::Afrinic),
            1u8 => Ok(Registry::Apnic),
            2u8 => Ok(Registry::Arin),
            3u8 => Ok(Registry::Iana),
            4u8 => Ok(Registry::Ietf),
            5u8 => Ok(Registry::Lacnic),
            6u8 => Ok(Registry::Ripencc),
            _ => Err(())
        }
    }

    pub fn index(&self) -> u8 {
        match *self {
            Registry::Afrinic => 0u8,
            Registry::Apnic => 1u8,
            Registry::Arin => 2u8,
            Registry::Iana => 3u8,
            Registry::Ietf => 4u8,
            Registry::Lacnic => 5u8,
            Registry::Ripencc => 6u8,
        }
    }

    pub fn description(&self) -> &str {
        match *self {
            Registry::Afrinic => "Africa Region",
            Registry::Apnic => "Asia/Pacific Region",
            Registry::Arin => "Canada, USA, and some Caribbean Islands",
            Registry::Iana => "Internet Assigned Numbers Authority(IANA)",
            Registry::Ietf => "Internet Engineering Task Force(IETF), Special Registry",
            Registry::Lacnic => "Latin America and some Caribbean Islands",
            Registry::Ripencc => "Europe, the Middle East, and Central Asia",
        }
    }
}

impl FromStr for Registry {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "afrinic" => Ok(Registry::Afrinic),
            "apnic" => Ok(Registry::Apnic),
            "arin" => Ok(Registry::Arin),
            "iana" => Ok(Registry::Iana),
            "ietf" => Ok(Registry::Ietf),
            "lacnic" => Ok(Registry::Lacnic),
            "ripencc" => Ok(Registry::Ripencc),
            e @ _ => Err(Error::ParseRegistryError(format!("Unknow Registry Name ({})", e))),
        }
    }
}

impl fmt::Display for Registry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Registry::Afrinic => write!(f, "afrinic"),
            Registry::Apnic => write!(f, "apnic"),
            Registry::Arin => write!(f, "arin"),
            Registry::Iana => write!(f, "iana"),
            Registry::Ietf => write!(f, "ietf"),
            Registry::Lacnic => write!(f, "lacnic"),
            Registry::Ripencc => write!(f, "ripencc"),
        }
    }
}
