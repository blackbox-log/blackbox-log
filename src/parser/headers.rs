mod frame_def;

pub use frame_def::{FieldDef, FrameDefs};

use super::{ParseError, ParseResult};
use crate::{LogVersion, Reader};
use bitter::BitReader;
use std::str;

#[derive(Debug)]
pub struct Headers {
    pub(crate) version: LogVersion,
    pub(crate) frames: FrameDefs,

    pub(crate) firmware_revision: String,
    pub(crate) board_info: String,
    pub(crate) craft_name: String,
}

impl Headers {
    pub fn parse(data: &mut Reader) -> ParseResult<Self> {
        let (name, _product) = parse_header(data)?;
        assert_eq!(name, "Product", "`Product` header must be first");

        let (name, version) = parse_header(data)?;
        assert_eq!(name, "Data version", "`Data version` header must be second");
        let version = version.parse().map_err(|_| ParseError::InvalidHeader {
            header: name,
            value: version,
        })?;

        let mut state = State::new(version);

        loop {
            if data.refill_lookahead() < 8 {
                return Err(ParseError::UnexpectedEof);
            }

            if data.peek(8) != b'H'.into() {
                break;
            }

            let (name, value) = parse_header(data)?;
            state.update(name, value)?;
        }

        state.finish()
    }
}

#[derive(Debug)]
struct State {
    version: LogVersion,
    frames: frame_def::Builders,
}

impl State {
    fn new(version: LogVersion) -> Self {
        Self {
            version,
            frames: frame_def::Builders::default(),
        }
    }

    fn update(&mut self, header: String, value: String) -> ParseResult<()> {
        match header {
            full_header if full_header.starts_with("Field ") => {
                let unknown_header = || ParseError::UnknownHeader(full_header.clone());

                let (frame, field) = full_header
                    .strip_prefix("Field ")
                    .unwrap()
                    .split_once(' ')
                    .ok_or_else(unknown_header)?;

                let frame = match frame {
                    "I" => &mut self.frames.intra,
                    "P" => &mut self.frames.inter,
                    "S" => &mut self.frames.slow,
                    _ => return Err(unknown_header()),
                };

                match field {
                    "name" => frame.names = Some(value),
                    "signed" => frame.signs = Some(value),
                    "width" => tracing::warn!("ignoring `{full_header}` header"),
                    "predictor" => frame.predictors = Some(value),
                    "encoding" => frame.encodings = Some(value),
                    _ => return Err(unknown_header()),
                };
            }
            header => tracing::warn!("skipping unknown header: `{header}` = `{value}`"),
        }

        Ok(())
    }

    fn finish(self) -> ParseResult<Headers> {
        todo!()
    }
}

/// Expects the next character to be the leading H
fn parse_header(data: &mut Reader) -> ParseResult<(String, String)> {
    match data.read_u8() {
        Some(b'H') => {}
        Some(_) => return Err(ParseError::Corrupted),
        None => return Err(ParseError::UnexpectedEof),
    }

    let mut line = Vec::new();
    while let Some(byte) = data.read_u8() {
        if byte == b'\n' {
            break;
        }

        line.push(byte);
    }

    let line = line.strip_prefix(&[b' ']).unwrap_or(&line);
    let line = str::from_utf8(line).unwrap();
    let (name, value) = line.split_once(':').unwrap();

    tracing::trace!("read header `{name}` = `{value}`");

    Ok((name.to_owned(), value.to_owned()))
}
