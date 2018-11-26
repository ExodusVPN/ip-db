

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ParseStatusError(String),
    ParseCountryError(String),
    ParseRegistryError(String),
    ParseRecordError(String),
}

