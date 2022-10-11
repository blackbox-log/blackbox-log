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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FrameKind {
    Event,
    Intra,
    Inter,
    Gps,
    GpsHome,
    Slow,
}

impl FrameKind {
    pub(crate) fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'E' => Some(Self::Event),
            b'I' => Some(Self::Intra),
            b'P' => Some(Self::Inter),
            b'G' => Some(Self::Gps),
            b'H' => Some(Self::GpsHome),
            b'S' => Some(Self::Slow),
            _ => None,
        }
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
