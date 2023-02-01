//! Types for the header section of blackbox logs.

use alloc::borrow::ToOwned;
use alloc::string::String;
use core::{fmt, str};

use hashbrown::HashMap;

use crate::frame::gps::{GpsFrameDef, GpsFrameDefBuilder};
use crate::frame::gps_home::{GpsHomeFrameDef, GpsHomeFrameDefBuilder};
use crate::frame::main::{MainFrameDef, MainFrameDefBuilder};
use crate::frame::slow::{SlowFrameDef, SlowFrameDefBuilder};
use crate::frame::{is_frame_def_header, parse_frame_def_header, DataFrameKind};
use crate::parser::{InternalError, InternalResult};
use crate::predictor::Predictor;
use crate::{Reader, Unit};

pub type ParseResult<T> = Result<T, ParseError>;

/// A fatal error encountered while parsing the headers of a log.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ParseError {
    /// The log uses a format version that is unsupported.
    UnsupportedVersion(String),
    /// The `Firmware revision` header could not be parsed.
    InvalidFirmware(String),
    /// Could not parse the value in header `header`.
    InvalidHeader { header: String, value: String },
    // TODO: include header
    /// Did not find a required header.
    MissingHeader,
    /// The file ended before the start of the data section.
    IncompleteHeaders,
    /// Definition for frame type `frame` is missing required a required field.
    MissingField { frame: DataFrameKind, field: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(v) => write!(f, "unsupported or invalid version: `{v}`"),
            Self::InvalidFirmware(firmware) => write!(f, "could not parse firmware: `{firmware}`"),
            Self::InvalidHeader { header, value } => {
                write!(f, "invalid value for header `{header}`: `{value}`")
            }
            Self::MissingHeader => {
                write!(f, "one or more headers required for parsing are missing")
            }
            Self::IncompleteHeaders => write!(f, "end of file found before data section"),
            Self::MissingField { frame, field } => {
                write!(f, "missing field `{field}` in `{frame}` frame definition")
            }
        }
    }
}

// TODO: waiting on https://github.com/rust-lang/rust-clippy/pull/9545 to land
#[allow(clippy::std_instead_of_core)]
#[cfg(feature = "std")]
impl std::error::Error for ParseError {}

/// Decoded headers containing metadata for a blackbox log.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct Headers<'data> {
    /// The format version of the log.
    pub version: LogVersion,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub main_frame_def: MainFrameDef<'data>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub slow_frame_def: SlowFrameDef<'data>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub gps_frame_def: Option<GpsFrameDef<'data>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) gps_home_frame_def: Option<GpsHomeFrameDef<'data>>,

    /// The full `Firmware revision` header.
    pub firmware_revision: &'data str,
    /// The firmware that wrote the log.
    pub firmware: Firmware,
    pub board_info: Option<&'data str>,
    pub craft_name: Option<&'data str>,

    /// The battery voltage measured at arm.
    pub(crate) vbat_reference: Option<u16>,
    /// Calibration for the accelerometer.
    pub(crate) acceleration_1g: Option<u16>,
    /// Calibration for the gyro in radians / second.
    pub(crate) gyro_scale: Option<f32>,

    pub(crate) min_throttle: Option<u16>,
    pub(crate) motor_output_range: Option<MotorOutputRange>,

    /// Any unknown headers with unparsed values
    pub unknown: HashMap<&'data str, &'data str>,
}

impl<'data> Headers<'data> {
    /// Parses only the headers of a blackbox log.
    ///
    /// `data` will be advanced to the start of the data section of the log,
    /// ready to be passed to [`DataParser::new`][`crate::DataParser::new`].
    ///
    /// **Note:** This assumes that `data` is aligned to the start of a log.
    pub fn parse(data: &mut Reader<'data>) -> ParseResult<Self> {
        // Skip product header
        let product = data.read_line();
        debug_assert_eq!(crate::MARKER.strip_suffix(&[b'\n']), product);

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
            };

            if !state.update(name, value) {
                return Err(ParseError::InvalidHeader {
                    header: name.to_owned(),
                    value: value.to_owned(),
                });
            }
        }

        state.finish()
    }

    fn validate(&self) -> ParseResult<()> {
        let has_accel = self.acceleration_1g.is_some();
        let has_min_throttle = self.min_throttle.is_some();
        // TODO: also check it is in a main frame
        let has_motor_0 = self.main_frame_def.has_motor_0();
        let has_vbat_ref = self.vbat_reference.is_some();
        let has_min_motor = self.motor_output_range.is_some();
        let has_gps_home = self.gps_home_frame_def.is_some();

        let predictor = |field, predictor| {
            let ok = match predictor {
                Predictor::MinThrottle => has_min_throttle,
                Predictor::Motor0 => has_motor_0,
                Predictor::HomeLat | Predictor::HomeLon => has_gps_home,
                Predictor::VBatReference => has_vbat_ref,
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
            let ok = if unit == Unit::Acceleration {
                has_accel
            } else {
                true
            };

            if ok {
                Ok(())
            } else {
                tracing::error!(field, ?unit, "missing required headers");
                Err(ParseError::MissingHeader)
            }
        };

        self.main_frame_def.validate(predictor, unit)?;
        self.slow_frame_def.validate(predictor, unit)?;

        if let Some(ref def) = self.gps_frame_def {
            def.validate(predictor, unit)?;
        }

        if let Some(ref def) = self.gps_home_frame_def {
            def.validate(predictor, unit)?;
        }

        Ok(())
    }
}

/// A supported log format version. (`Data version` header)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub enum LogVersion {
    V2,
}

/// A supported firmware.
///
/// This is not the same as the `Firmware type` header since all modern
/// firmwares set that to `Cleanflight`. This is instead decoded from `Firmware
/// revision`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Firmware {
    /// [Betaflight](https://github.com/betaflight/betaflight/)
    Betaflight(FirmwareVersion),
    /// [INAV](https://github.com/iNavFlight/inav/)
    Inav(FirmwareVersion),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl FirmwareVersion {
    fn from_str(s: &str) -> Option<Self> {
        let mut components = s.splitn(3, '.').map(|s| s.parse().ok());

        let major = components.next()??;
        let minor = components.next()??;
        let patch = components.next()??;

        Some(Self {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for FirmwareVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub(crate) struct MotorOutputRange {
    pub(crate) min: u16,
    #[allow(dead_code)]
    pub(crate) max: u16,
}

impl MotorOutputRange {
    pub(crate) fn from_str(s: &str) -> Option<Self> {
        s.split_once(',').and_then(|(min, max)| {
            let min = min.parse().ok()?;
            let max = max.parse().ok()?;
            Some(Self { min, max })
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
    firmware_kind: Option<&'data str>,
    board_info: Option<&'data str>,
    craft_name: Option<&'data str>,

    vbat_reference: Option<u16>,
    acceleration_1g: Option<u16>,
    gyro_scale: Option<f32>,

    min_throttle: Option<u16>,
    motor_output_range: Option<MotorOutputRange>,

    unknown: HashMap<&'data str, &'data str>,
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
            acceleration_1g: None,
            gyro_scale: None,

            min_throttle: None,
            motor_output_range: None,

            unknown: HashMap::new(),
        }
    }

    /// Returns `true` if the header/value pair was valid
    fn update(&mut self, header: &'data str, value: &'data str) -> bool {
        // TODO: try block
        (|| -> Result<(), ()> {
            match header {
                "Data version" => {
                    if value == "2" {
                        self.version = Some(LogVersion::V2);
                    } else {
                        return Err(());
                    }
                }
                "Firmware revision" => self.firmware_revision = Some(value),
                "Firmware type" => self.firmware_kind = Some(value),
                "Board information" => self.board_info = Some(value),
                "Craft name" => self.craft_name = Some(value),

                "vbatref" => {
                    let vbat_reference = value.parse().map_err(|_| ())?;
                    self.vbat_reference = Some(vbat_reference);
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

                    let scale = f32::from_bits(scale);
                    self.gyro_scale = Some(scale.to_radians());
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

                // Legacy calibration headers
                "vbatscale" | "vbat_scale" | "currentMeter" | "currentSensor" => {}

                header => {
                    tracing::debug!("skipping unknown header: `{header}` = `{value}`");
                    self.unknown.insert(header, value);
                }
            };

            Ok(())
        })()
        .is_ok()
    }

    fn finish(self) -> ParseResult<Headers<'data>> {
        let not_empty = |s: &&str| !s.is_empty();

        let firmware_revision = self.firmware_revision.ok_or(ParseError::MissingHeader)?;
        let firmware = parse_firmware(firmware_revision)?;

        // TODO: log where each error comes from
        let headers = Headers {
            version: self.version.ok_or(ParseError::MissingHeader)?,
            main_frame_def: self.main_frames.parse()?,
            slow_frame_def: self.slow_frames.parse()?,
            gps_frame_def: self.gps_frames.parse()?,
            gps_home_frame_def: self.gps_home_frames.parse()?,

            firmware_revision,
            firmware,
            board_info: self.board_info.map(str::trim).filter(not_empty),
            craft_name: self.craft_name.map(str::trim).filter(not_empty),

            vbat_reference: self.vbat_reference,
            acceleration_1g: self.acceleration_1g,
            gyro_scale: self.gyro_scale,

            min_throttle: self.min_throttle,
            motor_output_range: self.motor_output_range,

            unknown: self.unknown,
        };

        headers.validate()?;

        Ok(headers)
    }
}

fn parse_firmware(firmware_revision: &str) -> Result<Firmware, ParseError> {
    let mut iter = firmware_revision.split(' ');

    let invalid_fw = || Err(ParseError::InvalidFirmware(firmware_revision.to_owned()));

    let kind = iter.next().map(str::to_ascii_lowercase);
    let Some(version) = iter.next().and_then(FirmwareVersion::from_str) else {
        return invalid_fw();
    };

    match kind.as_deref() {
        Some("betaflight") => Ok(Firmware::Betaflight(version)),
        Some("inav") => Ok(Firmware::Inav(version)),
        Some("emuflight") => {
            tracing::error!("EmuFlight is not supported");
            invalid_fw()
        }
        _ => {
            tracing::error!("Could not parse firmware revision");
            invalid_fw()
        }
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
