use alloc::borrow::ToOwned;
use core::str::{self, FromStr};

use super::frame::{
    is_frame_def_header, parse_frame_def_header, FrameKind, GpsFrameDef, GpsFrameDefBuilder,
    MainFrameDef, MainFrameDefBuilder, MainUnit, SlowFrameDef, SlowFrameDefBuilder, SlowUnit,
};
use super::reader::ByteReader;
use super::{ParseError, ParseResult, Reader};
use crate::common::{FirmwareKind, LogVersion};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Headers<'data> {
    pub version: LogVersion,
    pub(crate) gps_frames: Option<GpsFrameDef<'data>>,
    pub(crate) main_frames: MainFrameDef<'data>,
    pub(crate) slow_frames: SlowFrameDef<'data>,

    pub firmware_revision: &'data str,
    pub firmware_kind: FirmwareKind,
    pub board_info: &'data str,
    pub craft_name: &'data str,

    /// Measured battery voltage at arm
    pub vbat_reference: u16,
    pub vbat_scale: u16,
    pub current_meter: CurrentMeterConfig,

    pub acceleration_1g: u16,
    pub gyro_scale: f32,

    pub min_throttle: u16,
    pub motor_output_range: MotorOutputRange,
}

impl<'data> Headers<'data> {
    pub(crate) fn main_fields(&self) -> impl Iterator<Item = (&str, MainUnit)> {
        self.main_frames.iter()
    }

    pub(crate) fn slow_fields(&self) -> impl Iterator<Item = (&str, SlowUnit)> {
        self.slow_frames.iter()
    }

    pub(crate) fn parse(data: &mut Reader<'data>) -> ParseResult<Self> {
        let bytes = &mut data.bytes();

        check_product(bytes)?;
        let version = get_version(bytes)?;

        let mut state = State::new(version);

        loop {
            match bytes.peek() {
                Some(b'H') => {}
                Some(_) => break,
                None => return Err(ParseError::UnexpectedEof),
            }

            let (name, value) = parse_header(bytes)?;
            state.update(name, value).map_err(|e| {
                tracing::error!("state.update error: {e}");
                e
            })?;
        }

        state.finish()
    }
}

fn check_product(bytes: &mut ByteReader) -> Result<(), ParseError> {
    let (product, _) = parse_header(bytes)?;
    if product.to_ascii_lowercase() != "product" {
        tracing::error!("`Product` header must be first");
        return Err(ParseError::Corrupted);
    };

    Ok(())
}

fn get_version(bytes: &mut ByteReader) -> Result<LogVersion, ParseError> {
    let (name, value) = parse_header(bytes)?;

    if name.to_ascii_lowercase() != "data version" {
        tracing::error!("`Data version` header must be second");
        return Err(ParseError::UnsupportedVersion(value.to_owned()));
    }

    value
        .parse()
        .map_err(|_| ParseError::UnsupportedVersion(value.to_owned()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CurrentMeterConfig {
    pub offset: u16,
    pub scale: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct MotorOutputRange(u16, u16);

impl MotorOutputRange {
    pub const fn new(min: u16, max: u16) -> Self {
        Self(min, max)
    }

    pub const fn min(&self) -> u16 {
        self.0
    }

    pub const fn max(&self) -> u16 {
        self.1
    }
}

impl FromStr for MotorOutputRange {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_once(',')
            .and_then(|(min, max)| {
                let min = min.parse().ok()?;
                let max = max.parse().ok()?;
                Some(MotorOutputRange::new(min, max))
            })
            .ok_or(ParseError::Corrupted)
    }
}

#[derive(Debug)]
struct State<'data> {
    version: LogVersion,
    gps_frames: GpsFrameDefBuilder<'data>,
    main_frames: MainFrameDefBuilder<'data>,
    slow_frames: SlowFrameDefBuilder<'data>,

    firmware_revision: Option<&'data str>,
    firmware_kind: Option<FirmwareKind>,
    board_info: Option<&'data str>,
    craft_name: Option<&'data str>,

    vbat_reference: Option<u16>,
    vbat_scale: Option<u16>,
    current_meter: Option<CurrentMeterConfig>,

    acceleration_1g: Option<u16>,
    gyro_scale: Option<f32>,

    min_throttle: Option<u16>,
    motor_output_range: Option<MotorOutputRange>,
}

impl<'data> State<'data> {
    fn new(version: LogVersion) -> Self {
        Self {
            version,
            gps_frames: GpsFrameDef::builder(),
            main_frames: MainFrameDef::builder(),
            slow_frames: SlowFrameDef::builder(),

            firmware_revision: None,
            firmware_kind: None,
            board_info: None,
            craft_name: None,

            vbat_reference: None,
            vbat_scale: None,
            current_meter: None,

            acceleration_1g: None,
            gyro_scale: None,

            min_throttle: None,
            motor_output_range: None,
        }
    }

    fn update(&mut self, header: &'data str, value: &'data str) -> ParseResult<()> {
        match header.to_ascii_lowercase().as_str() {
            "firmware revision" => self.firmware_revision = Some(value),
            "firmware type" => self.firmware_kind = Some(value.parse()?),
            "board information" => self.board_info = Some(value),
            "craft name" => self.craft_name = Some(value),

            "vbatref" => {
                let vbat_reference = value.parse().map_err(|_| ParseError::Corrupted)?;
                self.vbat_reference = Some(vbat_reference);
            }
            "vbatscale" | "vbat_scale" => {
                let vbat_scale = value.parse().map_err(|_| ParseError::Corrupted)?;
                self.vbat_scale = Some(vbat_scale);
            }
            "currentmeter" | "currentsensor" => {
                let (offset, scale) = value.split_once(',').ok_or(ParseError::Corrupted)?;
                let offset = offset.parse().map_err(|_| ParseError::Corrupted)?;
                let scale = scale.parse().map_err(|_| ParseError::Corrupted)?;

                self.current_meter = Some(CurrentMeterConfig { offset, scale });
            }
            "acc_1g" => {
                let one_g = value.parse().map_err(|_| ParseError::Corrupted)?;
                self.acceleration_1g = Some(one_g);
            }
            "gyro.scale" | "gyro_scale" => {
                let scale = if let Some(hex) = value.strip_prefix("0x") {
                    u32::from_str_radix(hex, 16).map_err(|_| ParseError::Corrupted)?
                } else {
                    value.parse().map_err(|_| ParseError::Corrupted)?
                };

                self.gyro_scale = Some(f32::from_bits(scale));
            }
            "minthrottle" => {
                let min_throttle = value.parse().map_err(|_| ParseError::Corrupted)?;
                self.min_throttle = Some(min_throttle);
            }
            "motoroutput" => {
                let range = value.parse().map_err(|_| ParseError::Corrupted)?;
                self.motor_output_range = Some(range);
            }

            _ if is_frame_def_header(header) => {
                let (frame_kind, property) = parse_frame_def_header(header).unwrap();

                match frame_kind {
                    FrameKind::Gps => self.gps_frames.update(property, value),
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
            gps_frames: self.gps_frames.parse()?,
            main_frames: self.main_frames.parse()?,
            slow_frames: self.slow_frames.parse()?,

            firmware_revision: self.firmware_revision.ok_or(ParseError::Corrupted)?,
            firmware_kind: self.firmware_kind.ok_or(ParseError::Corrupted)?,
            board_info: self.board_info.ok_or(ParseError::Corrupted)?,
            craft_name: self.craft_name.ok_or(ParseError::Corrupted)?,

            vbat_reference: self.vbat_reference.ok_or(ParseError::Corrupted)?,
            vbat_scale: self.vbat_scale.ok_or(ParseError::Corrupted)?,
            current_meter: self.current_meter.ok_or(ParseError::Corrupted)?,

            acceleration_1g: self.acceleration_1g.ok_or(ParseError::Corrupted)?,
            gyro_scale: self.gyro_scale.ok_or(ParseError::Corrupted)?,

            min_throttle: self.min_throttle.ok_or(ParseError::Corrupted)?,
            motor_output_range: self.motor_output_range.ok_or(ParseError::Corrupted)?,
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

    let line = str::from_utf8(line).map_err(|_| ParseError::Corrupted)?;
    let line = line.strip_prefix(' ').unwrap_or(line);
    let (name, value) = line.split_once(':').ok_or(ParseError::Corrupted)?;

    tracing::trace!("read header `{name}` = `{value}`");

    Ok((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Corrupted")]
    fn invalid_utf8() {
        let mut b = Reader::new(b"H \xFF:\xFF\n");
        parse_header(&mut b.bytes()).unwrap();
    }
}
