mod data;
pub mod decoders;
mod headers;
mod predictor;

pub use data::{Data, Event, Frame};
pub use decoders::Encoding;
pub use headers::{FieldDef, FrameDefs, Headers};
pub use predictor::Predictor;

use crate::{Log, Reader};

pub type ParseResult<T> = Result<T, ParseError>;

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
    #[error("invalid/corrupted data")]
    Corrupted,
    #[error("unexpected end of file")]
    UnexpectedEof,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Config {
    /// Skip applying predictors to the parsed values
    pub raw: bool,
}

impl Config {
    pub fn parse(&self, data: &[u8]) -> ParseResult<Log> {
        let mut data = Reader::new(data);
        let headers = Headers::parse(&mut data)?;
        let data = Data::parse(&mut data, self, &headers)?;

        Ok(Log { headers, data })
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
