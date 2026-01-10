use crate::scanner::{Marker, ScanError};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Message(String),
    MarkedMessage { msg: String, mark: Marker },
    UnsupportedType(&'static str),
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl From<ScanError> for Error {
    fn from(error: ScanError) -> Self {
        Error::Message(error.to_string())
    }
}

impl std::ops::Add<Marker> for Error {
    type Output = Self;

    fn add(self, mark: Marker) -> Self {
        match self {
            Self::Message(msg) => Self::MarkedMessage { msg, mark },
            _ => self,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::MarkedMessage { msg, mark } => {
                write!(
                    formatter,
                    "{} at line {} column {}",
                    msg,
                    mark.line(),
                    mark.col()
                )
            }
            Error::UnsupportedType(t) => {
                write!(formatter, "{} (de)serialization is not supported", t)
            }
        }
    }
}

impl std::error::Error for Error {}
