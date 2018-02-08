
use std::str::FromStr;
use std::fmt;

use error::Error;


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd)]
pub enum Status {
    Allocated,
    Assigned,
    Available,
    Reserved,
}

impl FromStr for Status {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "allocated" => Ok(Status::Allocated),
            "assigned" => Ok(Status::Assigned),
            "available" => Ok(Status::Available),
            "reserved" => Ok(Status::Reserved),
            e @ _ => Err(Error::ParseStatusError(format!("Unknow Status ({})", e))),
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Status::Allocated => write!(f, "allocated"),
            Status::Assigned => write!(f, "assigned"),
            Status::Available => write!(f, "available"),
            Status::Reserved => write!(f, "reserved"),
        }
    }
}
