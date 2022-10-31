mod data;
pub mod decode;
mod frame;
pub mod headers;
mod predictor;
mod reader;

use alloc::string::String;
use core::fmt;

pub use self::data::{Data, Event};
pub use self::decode::Encoding;
pub use self::frame::{MainFrame, MainUnit, MainValue, SlowFrame, SlowUnit, SlowValue};
pub use self::headers::Headers;
pub use self::predictor::Predictor;
pub use self::reader::Reader;

pub type ParseResult<T> = Result<T, ParseError>;
pub(crate) const MARKER: &[u8] = b"H Product:Blackbox flight data recorder by Nicholas Sherlock\n";

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(u8)]
    enum FrameKind {
        Event = b'E',
        Intra = b'I',
        Inter = b'P',
        Gps = b'G',
        GpsHome = b'H',
        Slow = b'S',
    }
}

#[allow(clippy::cast_possible_wrap)]
#[inline(always)]
pub(crate) const fn as_signed(x: u32) -> i32 {
    x as i32
}

#[allow(clippy::cast_sign_loss)]
#[inline(always)]
pub(crate) const fn as_unsigned(x: i32) -> u32 {
    x as u32
}

pub(crate) fn to_base_field(field: &str) -> &str {
    field.split_once('[').map_or(field, |(base, _)| base)
}
