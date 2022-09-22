mod data;
pub mod decode;
mod frame;
mod headers;
mod predictor;
mod reader;

pub use data::{Data, Event};
pub use decode::Encoding;
pub use frame::{Frame, MainFrame, SlowFrame};
pub use headers::Headers;
pub use predictor::Predictor;
pub use reader::Reader;

use crate::Log;
use std::str::Utf8Error;

pub type ParseResult<T> = Result<T, ParseError>;
pub(crate) const MARKER: &[u8] = b"H Product:Blackbox flight data recorder by Nicholas Sherlock\n";

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unknown product name: `{0}`")]
    UnknownProduct(String),
    #[error("unsupported version: `{0}`")]
    UnsupportedVersion(String),
    #[error("invalid value for header `{header}`: `{value}`")]
    InvalidHeader { header: String, value: String },
    #[error("unknown or invalid header name: `{0}`")]
    UnknownHeader(String),
    #[error("header with invalid UTF-8 bytes")]
    HeaderInvalidUtf8(#[from] Utf8Error),
    #[error("header without a colon as separator")]
    HeaderMissingColon,
    #[error("missing header `{0}`")]
    MissingHeader(String),
    #[error("missing field `{0}`")]
    MissingField(&'static str),
    #[error("invalid/corrupted data")]
    Corrupted,
    #[error("unexpected end of file")]
    UnexpectedEof,
}

impl ParseError {
    pub(crate) fn invalid_header(header: impl Into<String>, value: impl Into<String>) -> Self {
        Self::InvalidHeader {
            header: header.into(),
            value: value.into(),
        }
    }
}

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
