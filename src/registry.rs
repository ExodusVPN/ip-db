
use std::str::FromStr;
use std::fmt;

use error::Error;


/// Number Resources
/// 
/// `https://www.iana.org/numbers`
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Registry {
    /// Africa Region
    Afrinic,
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
