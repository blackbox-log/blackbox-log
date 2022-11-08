// Need it `pub` to run benchmarks and fuzz test
#[doc(hidden)]
pub mod decode;

mod data;
mod frame;
pub mod headers;
mod predictor;
mod reader;

use alloc::string::String;
use core::fmt;

pub(crate) use self::data::Data;
pub use self::data::{Event, Stats};
pub use self::decode::Encoding;
pub(crate) use self::frame::{GpsHomeFrame, MainFrame, SlowFrame};
pub use self::frame::{Unit, Value};
pub use self::headers::Headers;
pub(crate) use self::predictor::Predictor;
pub use self::reader::Reader;

pub type ParseResult<T> = Result<T, ParseError>;
pub(crate) const MARKER: &[u8] = b"H Product:Blackbox flight data recorder by Nicholas Sherlock\n";

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ParseError {
    UnsupportedVersion(String),
    UnknownFirmware(String),
    MissingHeader,
    Corrupted,
    UnexpectedEof,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(v) => write!(f, "unsupported or invalid version: `{v}`"),
            Self::UnknownFirmware(firmware) => write!(f, "unknown firmware: `{firmware}`"),
            Self::MissingHeader => {
                write!(f, "one or more headers required for parsing are missing")
            }
            Self::Corrupted => write!(f, "invalid/corrupted data"),
            Self::UnexpectedEof => write!(f, "unexpected end of file"),
        }
    }
}

// TODO: waiting on https://github.com/rust-lang/rust-clippy/pull/9545 to land
#[allow(clippy::std_instead_of_core)]
#[cfg(feature = "std")]
impl std::error::Error for ParseError {}

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(u8)]
    pub(crate) enum FrameKind {
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
