use super::frame::{
    is_frame_def_header, parse_frame_def_header, FrameKind, MainFrameDef, MainFrameDefBuilder,
    SlowFrameDef, SlowFrameDefBuilder,
};
use super::reader::ByteReader;
use super::{ParseError, ParseResult, Reader};
use crate::LogVersion;
use std::iter;
use std::str::{self, FromStr};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Headers<'data> {
    pub version: LogVersion,
    pub(crate) main_frames: MainFrameDef<'data>,
    pub(crate) slow_frames: SlowFrameDef<'data>,

    pub firmware_revision: &'data str,
    pub firmware_kind: FirmwareKind,
    pub board_info: &'data str,
    pub craft_name: &'data str,

    /// Measured battery voltage at arm
    pub vbat_reference: u16,
}

impl<'data> Headers<'data> {
    pub fn main_fields(&self) -> impl Iterator<Item = &str> {
        iter::once(self.main_frames.iteration.name)
            .chain(iter::once(self.main_frames.time.name))
            .chain(self.main_frames.fields.iter().map(|f| f.name))
    }

    pub fn slow_fields(&self) -> impl Iterator<Item = &str> {
        self.slow_frames.0.iter().map(|f| f.name)
    }

    pub(crate) fn parse(data: &mut Reader<'data>) -> ParseResult<Self> {
        let bytes = &mut data.bytes();

        let (name, _product) = parse_header(bytes)?;
        assert_eq!(name, "Product", "`Product` header must be first");

        let (name, version) = parse_header(bytes)?;
        assert_eq!(name, "Data version", "`Data version` header must be second");
        let version = version.parse().map_err(|_| ParseError::InvalidHeader {
            header: name.to_owned(),
            value: version.to_owned(),
        })?;

        let mut state = State::new(version);

        loop {
            match bytes.peek() {
                Some(b'H') => {}
                Some(_) => break,
                None => return Err(ParseError::UnexpectedEof),
            }

            let (name, value) = parse_header(bytes)?;
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
struct State<'data> {
    version: LogVersion,
    main_frames: MainFrameDefBuilder<'data>,
    slow_frames: SlowFrameDefBuilder<'data>,

    firmware_revision: Option<&'data str>,
    firmware_kind: Option<FirmwareKind>,
    board_info: Option<&'data str>,
    craft_name: Option<&'data str>,

    vbat_reference: Option<u16>,
}

impl<'data> State<'data> {
    fn new(version: LogVersion) -> Self {
        Self {
            version,
            main_frames: MainFrameDef::builder(),
            slow_frames: SlowFrameDef::builder(),

            firmware_revision: None,
            firmware_kind: None,
            board_info: None,
            craft_name: None,
            vbat_reference: None,
        }
    }

    fn update(&mut self, header: &'data str, value: &'data str) -> ParseResult<()> {
        match header {
            "Firmware revision" => self.firmware_revision = Some(value),
            "Firmware type" => self.firmware_kind = Some(value.parse()?),
            "Board information" => self.board_info = Some(value),
            "Craft name" => self.craft_name = Some(value),
            "vbatref" => {
                self.vbat_reference =
                    Some(value.parse().map_err(|_| ParseError::InvalidHeader {
                        header: header.to_owned(),
                        value: value.to_owned(),
                    })?);
            }
            _ if is_frame_def_header(header) => {
                let (frame_kind, property) = parse_frame_def_header(header).unwrap();

                match frame_kind {
                    FrameKind::Inter | FrameKind::Intra => {
                        self.main_frames.update(frame_kind, property, value);
                    }
                    FrameKind::Slow => self.slow_frames.update(property, value),
                }
            }
            header => tracing::debug!("skipping unknown header: `{header}` = `{value}`"),
        }

        Ok(())
    }

    fn finish(self) -> ParseResult<Headers<'data>> {
        Ok(Headers {
            version: self.version,
            main_frames: self.main_frames.parse()?,
            slow_frames: self.slow_frames.parse()?,

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
fn parse_header<'data>(bytes: &mut ByteReader<'data, '_>) -> ParseResult<(&'data str, &'data str)> {
    match bytes.read_u8() {
        Some(b'H') => {}
        Some(_) => return Err(ParseError::Corrupted),
        None => return Err(ParseError::UnexpectedEof),
    }

    let line = bytes.read_line().ok_or(ParseError::UnexpectedEof)?;

    let line = str::from_utf8(line)?;
    let line = line.strip_prefix(' ').unwrap_or(line);
    let (name, value) = line.split_once(':').ok_or(ParseError::HeaderMissingColon)?;

    tracing::trace!("read header `{name}` = `{value}`");

    Ok((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "HeaderInvalidUtf8")]
    fn invalid_utf8() {
        let mut b = Reader::new(b"H \xFF:\xFF\n");
        parse_header(&mut b.bytes()).unwrap();
    }
}
