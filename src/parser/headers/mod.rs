mod frame_def;

pub use frame_def::{FieldDef, FrameDef, FrameDefs};

use super::{ParseError, ParseResult};
use crate::{LogVersion, Reader};
use bitter::BitReader;
use std::str;
use std::str::FromStr;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Headers {
    pub(crate) version: LogVersion,
    pub(crate) frames: FrameDefs,

    pub(crate) firmware_revision: String,
    pub(crate) firmware_kind: FirmwareKind,
    pub(crate) board_info: String,
    pub(crate) craft_name: String,

    /// Measured battery voltage at arm
    pub(crate) vbat_reference: u16,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FirmwareKind {
    Baseflight,
    Cleanflight,
}

impl FromStr for FirmwareKind {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Cleanflight" => Ok(Self::Cleanflight),
            "Baseflight" => Ok(Self::Baseflight),
            _ => Err(ParseError::InvalidHeader {
                header: "Firmware type".to_owned(),
                value: s.to_owned(),
            }),
        }
    }
}

#[derive(Debug)]
struct State {
    version: LogVersion,
    frames: frame_def::FrameDefsBuilder,

    firmware_revision: Option<String>,
    firmware_kind: Option<FirmwareKind>,
    board_info: Option<String>,
    craft_name: Option<String>,

    vbat_reference: Option<u16>,
}

impl State {
    fn new(version: LogVersion) -> Self {
        Self {
            version,
            frames: frame_def::FrameDefs::builder(),
            firmware_revision: None,
            firmware_kind: None,
            board_info: None,
            craft_name: None,
            vbat_reference: None,
        }
    }

    fn update(&mut self, header: String, value: String) -> ParseResult<()> {
        match header.as_str() {
            "Firmware revision" => self.firmware_revision = Some(value),
            "Firmware type" => self.firmware_kind = Some(value.parse()?),
            "Board information" => self.board_info = Some(value),
            "Craft name" => self.craft_name = Some(value),
            "vbatref" => {
                self.vbat_reference = Some(
                    value
                        .parse()
                        .map_err(|_| ParseError::InvalidHeader { header, value })?,
                );
            }
            _ if header.starts_with("Field ") => {
                let unknown_header = || ParseError::UnknownHeader(header.clone());

                let (frame, field) = header
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
                    "width" => tracing::warn!("ignoring `{header}` header"),
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
        Ok(Headers {
            version: self.version,
            frames: self.frames.parse()?,

            // TODO: return an error instead of unwrap
            firmware_revision: self.firmware_revision.unwrap(),
            firmware_kind: self.firmware_kind.unwrap(),
            board_info: self.board_info.unwrap(),
            craft_name: self.craft_name.unwrap(),
            vbat_reference: self.vbat_reference.unwrap(),
        })
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
