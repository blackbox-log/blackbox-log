mod data;
pub mod decoders;
mod headers;
mod predictor;

pub use data::{Data, Event, Frame, FrameKind};
pub use decoders::Encoding;
pub use headers::{FieldDef, FrameDefs, Headers};
pub use predictor::Predictor;

use crate::{Log, Reader};

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

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Default, Clone, Copy)]
pub struct Config {
    /// Skip applying predictors to the parsed values
    raw: bool,
}

impl Config {
    pub fn parse(&self, data: &[u8]) -> ParseResult<Log> {
        let mut data = Reader::new(data);
        let headers = Headers::parse(&mut data)?;
        let data = Data::parse(&mut data, &headers)?;

        Ok(Log { headers, data })
    }
}
