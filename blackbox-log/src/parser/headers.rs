use alloc::borrow::ToOwned;
use core::str;

use super::frame::{
    is_frame_def_header, parse_frame_def_header, DataFrameKind, GpsFrameDef, GpsFrameDefBuilder,
    GpsHomeFrameDef, GpsHomeFrameDefBuilder, MainFrameDef, MainFrameDefBuilder, MainUnit,
    SlowFrameDef, SlowFrameDefBuilder, SlowUnit,
};
use super::{InternalError, InternalResult, ParseError, ParseResult, Predictor, Reader, Unit};
use crate::common::{FirmwareKind, LogVersion};

#[allow(dead_code)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Headers<'data> {
    pub version: LogVersion,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) main_frames: MainFrameDef<'data>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) slow_frames: SlowFrameDef<'data>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) gps_frames: Option<GpsFrameDef<'data>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) gps_home_frames: Option<GpsHomeFrameDef<'data>>,

    pub firmware_revision: &'data str,
    pub firmware_kind: FirmwareKind,
    pub board_info: Option<&'data str>,
    pub craft_name: Option<&'data str>,

    pub vbat: Option<VbatConfig>,
    pub current_meter: Option<CurrentMeterConfig>,

    pub acceleration_1g: Option<u16>,
    pub gyro_scale: Option<f32>,

    pub min_throttle: Option<u16>,
    pub motor_output_range: Option<MotorOutputRange>,
}

impl<'data> Headers<'data> {
    pub(crate) fn main_fields(&self) -> impl Iterator<Item = (&str, MainUnit)> {
        self.main_frames.iter()
    }

    pub(crate) fn slow_fields(&self) -> impl Iterator<Item = (&str, SlowUnit)> {
        self.slow_frames.iter()
    }

    pub(crate) fn parse(data: &mut Reader<'data>) -> ParseResult<Self> {
        // Skip product header
        let product = data.read_line();
        debug_assert_eq!(Some(super::MARKER.strip_suffix(&[b'\n']).unwrap()), product);

        let mut state = State::new();

        loop {
            if data.peek() != Some(b'H') {
                break;
            }

            let restore = data.get_restore_point();
            let (name, value) = match parse_header(data) {
                Ok(x) => x,
                Err(InternalError::Retry) => {
                    tracing::debug!("found corrupted header");
                    data.restore(restore);
                    break;
                }
                Err(InternalError::Eof) => return Err(ParseError::IncompleteHeaders),
                Err(InternalError::Fatal(err)) => return Err(err),
            };

            if !state.update(name, value) {
                return Err(ParseError::InvalidHeader(name.to_owned(), value.to_owned()));
            }
        }

        state.finish()
    }

    fn validate(&self) -> ParseResult<()> {
        let has_accel = self.acceleration_1g.is_some();
        let has_current_meter = self.current_meter.is_some();
        let has_min_throttle = self.min_throttle.is_some();
        let has_motor_0 = self.main_frames.has_motor_0();
        let has_vbat = self.vbat.is_some();
        let has_min_motor = self.motor_output_range.is_some();
        let has_gps_home = self.gps_home_frames.is_some();

        let predictor = |field, predictor| {
            let ok = match predictor {
                Predictor::MinThrottle => has_min_throttle,
                Predictor::Motor0 => has_motor_0,
                Predictor::HomeLat => has_gps_home,
                Predictor::VBatReference => has_vbat,
                Predictor::MinMotor => has_min_motor,
                Predictor::Zero
                | Predictor::Previous
                | Predictor::StraightLine
                | Predictor::Average2
                | Predictor::Increment
                | Predictor::FifteenHundred
                | Predictor::LastMainFrameTime => true,
            };

            if ok {
                Ok(())
            } else {
                tracing::error!(field, ?predictor, "missing required headers");
                Err(ParseError::MissingHeader)
            }
        };

        let unit = |field, unit| {
            let ok = match unit {
                Unit::Amperage => has_current_meter,
                Unit::Voltage => has_vbat,
                Unit::Acceleration => has_accel,
                Unit::FrameTime
                | Unit::Rotation
                | Unit::FlightMode
                | Unit::State
                | Unit::FailsafePhase
                | Unit::Boolean
                | Unit::Unitless => true,
            };

            if ok {
                Ok(())
            } else {
                tracing::error!(field, ?unit, "missing required headers");
                Err(ParseError::MissingHeader)
            }
        };

        self.main_frames.validate(predictor, unit)?;
        self.slow_frames.validate(predictor, unit)?;

        if let Some(ref def) = self.gps_frames {
            def.validate(predictor, unit)?;
        }

        if let Some(ref def) = self.gps_home_frames {
            def.validate(predictor, unit)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VbatConfig {
    /// Measured battery voltage at arm
    pub reference: u16,
    pub scale: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CurrentMeterConfig {
    pub offset: u16,
    pub scale: u16,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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

    pub(crate) fn from_str(s: &str) -> Option<Self> {
        s.split_once(',').and_then(|(min, max)| {
            let min = min.parse().ok()?;
            let max = max.parse().ok()?;
            Some(MotorOutputRange::new(min, max))
        })
    }
}

#[derive(Debug)]
struct State<'data> {
    version: Option<LogVersion>,
    main_frames: MainFrameDefBuilder<'data>,
    slow_frames: SlowFrameDefBuilder<'data>,
    gps_frames: GpsFrameDefBuilder<'data>,
    gps_home_frames: GpsHomeFrameDefBuilder<'data>,

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
    fn new() -> Self {
        Self {
            version: None,
            main_frames: MainFrameDef::builder(),
            slow_frames: SlowFrameDef::builder(),
            gps_frames: GpsFrameDef::builder(),
            gps_home_frames: GpsHomeFrameDef::builder(),

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

    /// Returns `true` if the header/value pair was valid
    fn update(&mut self, header: &'data str, value: &'data str) -> bool {
        // TODO: try block
        (|| -> Result<(), ()> {
            match header {
                "Data version" => self.version = Some(value.parse().map_err(|_| ())?),
                "Firmware revision" => self.firmware_revision = Some(value),
                "Firmware type" => self.firmware_kind = Some(value.parse().map_err(|_| ())?),
                "Board information" => self.board_info = Some(value),
                "Craft name" => self.craft_name = Some(value),

                "vbatref" => {
                    let vbat_reference = value.parse().map_err(|_| ())?;
                    self.vbat_reference = Some(vbat_reference);
                }
                "vbatscale" | "vbat_scale" => {
                    let vbat_scale = value.parse().map_err(|_| ())?;
                    self.vbat_scale = Some(vbat_scale);
                }
                "currentMeter" | "currentSensor" => {
                    let (offset, scale) = value.split_once(',').ok_or(())?;
                    let offset = offset.parse().map_err(|_| ())?;
                    let scale = scale.parse().map_err(|_| ())?;

                    self.current_meter = Some(CurrentMeterConfig { offset, scale });
                }
                "acc_1G" => {
                    let one_g = value.parse().map_err(|_| ())?;
                    self.acceleration_1g = Some(one_g);
                }
                "gyro.scale" | "gyro_scale" => {
                    let scale = if let Some(hex) = value.strip_prefix("0x") {
                        u32::from_str_radix(hex, 16).map_err(|_| ())?
                    } else {
                        value.parse().map_err(|_| ())?
                    };

                    self.gyro_scale = Some(f32::from_bits(scale));
                }
                "minthrottle" => {
                    let min_throttle = value.parse().map_err(|_| ())?;
                    self.min_throttle = Some(min_throttle);
                }
                "motorOutput" => {
                    let range = MotorOutputRange::from_str(value).ok_or(())?;
                    self.motor_output_range = Some(range);
                }

                _ if is_frame_def_header(header) => {
                    let (frame_kind, property) = parse_frame_def_header(header).unwrap();

                    match frame_kind {
                        DataFrameKind::Inter | DataFrameKind::Intra => {
                            self.main_frames.update(frame_kind, property, value);
                        }
                        DataFrameKind::Slow => self.slow_frames.update(property, value),
                        DataFrameKind::Gps => self.gps_frames.update(property, value),
                        DataFrameKind::GpsHome => self.gps_home_frames.update(property, value),
                    }
                }
                header => tracing::debug!("skipping unknown header: `{header}` = `{value}`"),
            };

            Ok(())
        })()
        .is_ok()
    }

    fn finish(self) -> ParseResult<Headers<'data>> {
        let not_empty = |s: &&str| !s.is_empty();

        let vbat = if let (Some(reference), Some(scale)) = (self.vbat_reference, self.vbat_scale) {
            Some(VbatConfig { reference, scale })
        } else {
            None
        };

        // TODO: log where each error comes from
        let headers = Headers {
            version: self.version.ok_or(ParseError::MissingHeader)?,
            main_frames: self.main_frames.parse()?,
            slow_frames: self.slow_frames.parse()?,
            gps_frames: self.gps_frames.parse()?,
            gps_home_frames: self.gps_home_frames.parse()?,

            firmware_revision: self.firmware_revision.ok_or(ParseError::MissingHeader)?,
            firmware_kind: self.firmware_kind.ok_or(ParseError::MissingHeader)?,
            board_info: self.board_info.map(str::trim).filter(not_empty),
            craft_name: self.craft_name.map(str::trim).filter(not_empty),

            vbat,
            current_meter: self.current_meter,

            acceleration_1g: self.acceleration_1g,
            gyro_scale: self.gyro_scale,

            min_throttle: self.min_throttle,
            motor_output_range: self.motor_output_range,
        };

        headers.validate()?;

        Ok(headers)
    }
}

/// Expects the next character to be the leading H
fn parse_header<'data>(bytes: &mut Reader<'data>) -> InternalResult<(&'data str, &'data str)> {
    match bytes.read_u8() {
        Some(b'H') => {}
        Some(_) => return Err(InternalError::Retry),
        None => return Err(InternalError::Eof),
    }

    let line = bytes.read_line().ok_or(InternalError::Eof)?;

    let line = str::from_utf8(line).map_err(|_| InternalError::Retry)?;
    let line = line.strip_prefix(' ').unwrap_or(line);
    let (name, value) = line.split_once(':').ok_or(InternalError::Retry)?;

    tracing::trace!("read header `{name}` = `{value}`");

    Ok((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Retry")]
    fn invalid_utf8() {
        let mut b = Reader::new(b"H \xFF:\xFF\n");
        parse_header(&mut b).unwrap();
    }
}
