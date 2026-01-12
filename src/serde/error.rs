use crate::{
    parser::Event,
    scanner::{Marker, ScanError},
};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Message(String),
    MarkedMessage {
        msg: String,
        mark: Marker,
    },
    UnsupportedType(&'static str),
    UnexpectedStreamStart {
        mark: Marker,
        expected: &'static str,
    },
    UnexpectedStreamEnd {
        mark: Marker,
        expected: &'static str,
    },
    UnexpectedDocumentStart {
        mark: Marker,
        expected: &'static str,
    },
    UnexpectedDocumentEnd {
        mark: Marker,
        expected: &'static str,
    },
    UnexpectedScalar {
        mark: Marker,
        expected: &'static str,
        value: String,
    },
    UnexpectedSequenceStart {
        mark: Marker,
        expected: &'static str,
    },
    UnexpectedSequenceEnd {
        mark: Marker,
        expected: &'static str,
    },
    UnexpectedMappingStart {
        mark: Marker,
        expected: &'static str,
    },
    UnexpectedMappingEnd {
        mark: Marker,
        expected: &'static str,
    },
}

impl Error {
    pub(crate) fn from_event(ev: Event, mark: Marker, expected: &'static str) -> Self {
        match ev {
            Event::StreamStart => Self::UnexpectedStreamStart { mark, expected },
            Event::StreamEnd => Self::UnexpectedStreamEnd { mark, expected },
            Event::DocumentStart => Self::UnexpectedDocumentStart { mark, expected },
            Event::DocumentEnd => Self::UnexpectedDocumentEnd { mark, expected },
            Event::Scalar(value, _, _) => Self::UnexpectedScalar {
                mark,
                expected,
                value,
            },
            Event::SequenceStart(_) => Self::UnexpectedSequenceStart { mark, expected },
            Event::SequenceEnd => Self::UnexpectedSequenceEnd { mark, expected },
            Event::MappingStart(_) => Self::UnexpectedMappingStart { mark, expected },
            Event::MappingEnd => Self::UnexpectedMappingEnd { mark, expected },
            _ => unreachable!(),
        }
    }
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
                write!(formatter, "{} at line {}", msg, mark.line())
            }
            Error::UnsupportedType(t) => {
                write!(formatter, "(de)serialization of {} is not supported", t)
            }
            Error::UnexpectedStreamStart { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the start of the stream at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedStreamEnd { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the end of the stream at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedDocumentStart { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the start of a document at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedDocumentEnd { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the end of a document at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedScalar {
                mark,
                expected,
                value,
            } => {
                write!(
                    formatter,
                    "expected {}, but found scalar \"{}\" at line {}",
                    expected,
                    value,
                    mark.line()
                )
            }
            Error::UnexpectedSequenceStart { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the start of a sequence at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedSequenceEnd { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the end of a sequence at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedMappingStart { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the start of a mapping at line {}",
                    expected,
                    mark.line()
                )
            }
            Error::UnexpectedMappingEnd { mark, expected } => {
                write!(
                    formatter,
                    "expected {}, but found the end of a mapping at line {}",
                    expected,
                    mark.line()
                )
            }
        }
    }
}

impl std::error::Error for Error {}
