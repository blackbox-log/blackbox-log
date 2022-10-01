mod data;
pub mod decode;
mod frame;
mod headers;
mod predictor;
mod reader;

use alloc::string::String;
use core::fmt;

pub use self::data::{Data, Event};
pub use self::decode::Encoding;
pub use self::frame::{Frame, MainFrame, SlowFrame};
pub use self::headers::Headers;
pub use self::predictor::Predictor;
pub use self::reader::Reader;
use crate::Log;

pub type ParseResult<T> = Result<T, ParseError>;
pub(crate) const MARKER: &[u8] = b"H Product:Blackbox flight data recorder by Nicholas Sherlock\n";

#[derive(Debug)]
pub enum ParseError {
    UnsupportedVersion(String),
    UnknownFirmware(String),
    Corrupted,
    UnexpectedEof,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(v) => write!(f, "unsupported or invalid version: `{v}`"),
            Self::UnknownFirmware(firmware) => write!(f, "unknown firmware: `{firmware}`"),
            Self::Corrupted => write!(f, "invalid/corrupted data"),
            Self::UnexpectedEof => write!(f, "unexpected end of file"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}

#[derive(Debug, Default, Clone, Copy)]
pub struct Config {
    /// Skip applying predictors to the parsed values
    pub raw: bool,
}

impl Config {
    pub fn parse<'data>(&self, data: &'data [u8]) -> ParseResult<Log<'data>> {
        Log::parse(self, data)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FrameKind {
    Event,
    Data(DataFrameKind),
}

impl FrameKind {
    pub(crate) fn from_byte(byte: u8) -> Option<Self> {
        if byte == b'E' {
            Some(Self::Event)
        } else {
            Some(Self::Data(DataFrameKind::from_byte(byte)?))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DataFrameKind {
    Intra,
    Inter,
    Gps,
    GpsHome,
    Slow,
}

impl DataFrameKind {
    pub(crate) fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'I' => Some(Self::Intra),
            b'P' => Some(Self::Inter),
            b'G' => Some(Self::Gps),
            b'H' => Some(Self::GpsHome),
            b'S' => Some(Self::Slow),
            _ => None,
        }
    }
}
