#[cfg(not(any(fuzzing, bench)))]
pub(crate) mod decode;
#[cfg(any(fuzzing, bench))]
pub mod decode;

use alloc::string::String;
use core::fmt;

pub(crate) use self::decode::Encoding;
use crate::frame::FrameKind;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ParseError {
    UnsupportedVersion(String),
    UnknownFirmware(String),
    InvalidHeader(String, String),
    // TODO: include header
    MissingHeader,
    IncompleteHeaders,
    MissingField(FrameKind, String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(v) => write!(f, "unsupported or invalid version: `{v}`"),
            Self::UnknownFirmware(firmware) => write!(f, "unknown firmware: `{firmware}`"),
            Self::InvalidHeader(header, value) => {
                write!(f, "invalid value for header `{header}`: `{value}`")
            }
            Self::MissingHeader => {
                write!(f, "one or more headers required for parsing are missing")
            }
            Self::IncompleteHeaders => write!(f, "end of file found before data section"),
            Self::MissingField(frame, field) => {
                write!(f, "missing field `{field}` in `{frame}` frame definition")
            }
        }
    }
}

// TODO: waiting on https://github.com/rust-lang/rust-clippy/pull/9545 to land
#[allow(clippy::std_instead_of_core)]
#[cfg(feature = "std")]
impl std::error::Error for ParseError {}

pub(crate) type InternalResult<T> = Result<T, InternalError>;

#[derive(Debug, Clone)]
pub(crate) enum InternalError {
    Fatal(ParseError),
    /// Found something unexpected, try to recover
    Retry,
    Eof,
}

impl From<ParseError> for InternalError {
    fn from(err: ParseError) -> Self {
        Self::Fatal(err)
    }
}

pub(crate) fn to_base_field(field: &str) -> &str {
    field.split_once('[').map_or(field, |(base, _)| base)
}
