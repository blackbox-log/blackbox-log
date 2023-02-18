//! Types for the header section of blackbox logs.

use alloc::borrow::ToOwned;
use alloc::string::String;
use core::str::FromStr;
use core::{fmt, str};

use hashbrown::HashMap;
use time::PrimitiveDateTime;

use crate::frame::gps::{GpsFrameDef, GpsFrameDefBuilder};
use crate::frame::gps_home::{GpsHomeFrameDef, GpsHomeFrameDefBuilder};
use crate::frame::main::{MainFrameDef, MainFrameDefBuilder};
use crate::frame::slow::{SlowFrameDef, SlowFrameDefBuilder};
use crate::frame::{is_frame_def_header, parse_frame_def_header, DataFrameKind};
use crate::parser::{InternalError, InternalResult};
use crate::predictor::Predictor;
use crate::utils::as_u32;
use crate::{Reader, Unit};

include_generated!("debug_mode");
include_generated!("disabled_fields");
include_generated!("features");
include_generated!("pwm_protocol");

pub type ParseResult<T> = Result<T, ParseError>;

/// A fatal error encountered while parsing the headers of a log.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "_serde", derive(serde::Serialize))]
pub enum ParseError {
    /// The log uses a format version that is unsupported or could not be
    /// parsed.
    UnsupportedVersion,
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
            Self::UnsupportedVersion => write!(f, "unsupported or invalid data version"),
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
#[non_exhaustive]
pub struct Headers<'data> {
    main_frame_def: MainFrameDef<'data>,
    slow_frame_def: SlowFrameDef<'data>,
    gps_frame_def: Option<GpsFrameDef<'data>>,
    gps_home_frame_def: Option<GpsHomeFrameDef<'data>>,

    firmware_revision: &'data str,
    pub(crate) firmware: InternalFirmware,
    firmware_date: Option<&'data str>,
    board_info: Option<&'data str>,
    craft_name: Option<&'data str>,

    debug_mode: DebugMode,
    disabled_fields: DisabledFields,
    features: FeatureSet,
    pwm_protocol: PwmProtocol,

    /// The battery voltage measured at arm.
    pub(crate) vbat_reference: Option<u16>,
    /// Calibration for the accelerometer.
    pub(crate) acceleration_1g: Option<u16>,
    /// Calibration for the gyro in radians / second.
    pub(crate) gyro_scale: Option<f32>,

    pub(crate) min_throttle: Option<u16>,
    pub(crate) motor_output_range: Option<MotorOutputRange>,

    unknown: HashMap<&'data str, &'data str>,
}

impl<'data> Headers<'data> {
    /// Parses only the headers of a blackbox log.
    ///
    /// `data` will be advanced to the start of the data section of the log,
    /// ready to be passed to [`DataParser::new`][crate::DataParser::new].
    ///
    /// **Note:** This assumes that `data` is aligned to the start of a log.
    pub fn parse(data: &mut Reader<'data>) -> ParseResult<Self> {
        // Skip product header
        let product = data.read_line();
        debug_assert_eq!(crate::MARKER.strip_suffix(&[b'\n']), product);
        let data_version = data.read_line();
        if !matches!(data_version, Some(b"H Data version:2")) {
            return Err(ParseError::UnsupportedVersion);
        }

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

/// Getters for various log headers.
impl<'data> Headers<'data> {
    #[inline]
    pub fn main_frame_def(&self) -> &MainFrameDef<'data> {
        &self.main_frame_def
    }

    #[inline]
    pub fn slow_frame_def(&self) -> &SlowFrameDef<'data> {
        &self.slow_frame_def
    }

    #[inline]
    pub fn gps_frame_def(&self) -> Option<&GpsFrameDef<'data>> {
        self.gps_frame_def.as_ref()
    }

    #[inline]
    pub(crate) fn gps_home_frame_def(&self) -> Option<&GpsHomeFrameDef<'data>> {
        self.gps_home_frame_def.as_ref()
    }

    /// The full `Firmware revision` header.
    ///
    /// Consider using the [`firmware`][Self::firmware] method instead.
    #[inline]
    pub fn firmware_revision(&self) -> &'data str {
        self.firmware_revision
    }

    /// The firmware that wrote the log.
    #[inline]
    pub fn firmware(&self) -> Firmware {
        self.firmware.into()
    }

    /// The `Firmware date` header
    pub fn firmware_date(&self) -> Option<Result<PrimitiveDateTime, &'data str>> {
        let format = time::macros::format_description!(
            "[month repr:short case_sensitive:false] [day padding:space] [year] [hour \
             repr:24]:[minute]:[second]"
        );
        self.firmware_date
            .map(|date| PrimitiveDateTime::parse(date, &format).map_err(|_| date))
    }

    /// The `Board info` header.
    #[inline]
    pub fn board_info(&self) -> Option<&'data str> {
        self.board_info
    }

    /// The `Craft name` header.
    #[inline]
    pub fn craft_name(&self) -> Option<&'data str> {
        self.craft_name
    }

    #[inline]
    pub fn debug_mode(&self) -> DebugMode {
        self.debug_mode
    }

    #[inline]
    pub fn disabled_fields(&self) -> DisabledFields {
        self.disabled_fields
    }

    #[inline]
    pub fn features(&self) -> FeatureSet {
        self.features
    }

    #[inline]
    pub fn pwm_protocol(&self) -> PwmProtocol {
        self.pwm_protocol
    }

    /// Any unknown headers.
    #[inline]
    pub fn unknown(&self) -> &HashMap<&'data str, &'data str> {
        &self.unknown
    }
}

/// A supported firmware.
///
/// This is not the same as the `Firmware type` header since all modern
/// firmwares set that to `Cleanflight`. This is instead decoded from `Firmware
/// revision`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "_serde", derive(serde::Serialize))]
pub enum Firmware {
    /// [Betaflight](https://github.com/betaflight/betaflight/)
    Betaflight(FirmwareVersion),
    /// [INAV](https://github.com/iNavFlight/inav/)
    Inav(FirmwareVersion),
}

impl Firmware {
    pub const fn name(&self) -> &'static str {
        match self {
            Firmware::Betaflight(_) => "Betaflight",
            Firmware::Inav(_) => "INAV",
        }
    }

    pub const fn version(&self) -> FirmwareVersion {
        let (Self::Betaflight(version) | Self::Inav(version)) = self;
        *version
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl FirmwareVersion {
    pub const fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(feature = "_serde")]
impl serde::Serialize for FirmwareVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[cfg(not(feature = "std"))]
        use alloc::string::ToString;
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum InternalFirmware {
    Betaflight4_2_0,
    Betaflight4_2_6,
    Betaflight4_2_8,
    Betaflight4_2_9,
    Betaflight4_2_11,
    Inav3_0_1,

    Betaflight4_3_0,
    Betaflight4_3_1,
    Inav5_0_0,
}

impl InternalFirmware {
    pub(crate) const fn is_betaflight(self) -> bool {
        match self {
            Self::Betaflight4_2_0
            | Self::Betaflight4_2_6
            | Self::Betaflight4_2_8
            | Self::Betaflight4_2_9
            | Self::Betaflight4_2_11
            | Self::Betaflight4_3_0
            | Self::Betaflight4_3_1 => true,
            Self::Inav3_0_1 | Self::Inav5_0_0 => false,
        }
    }

    pub(crate) const fn is_inav(self) -> bool {
        // Will need to be changed if any new firmwares are added
        !self.is_betaflight()
    }

    fn parse(revision: &str) -> ParseResult<Self> {
        let mut iter = revision.split(' ');

        let invalid_fw = || ParseError::InvalidFirmware(revision.to_owned());

        let kind = iter.next().map(str::to_ascii_lowercase);
        let version = iter.next().ok_or_else(invalid_fw)?;

        match kind.as_deref() {
            Some("betaflight") => match version {
                "4.2.0" => Ok(Self::Betaflight4_2_0),
                "4.2.6" => Ok(Self::Betaflight4_2_6),
                "4.2.8" => Ok(Self::Betaflight4_2_8),
                "4.2.9" => Ok(Self::Betaflight4_2_9),
                "4.2.11" => Ok(Self::Betaflight4_2_11),
                "4.3.0" => Ok(Self::Betaflight4_3_0),
                "4.3.1" => Ok(Self::Betaflight4_3_1),
                _ => Err(invalid_fw()),
            },
            Some("inav") => match version {
                "3.0.1" => Ok(Self::Inav3_0_1),
                "5.0.0" => Ok(Self::Inav5_0_0),
                _ => Err(invalid_fw()),
            },
            Some("emuflight") => {
                tracing::error!("EmuFlight is not supported");
                Err(invalid_fw())
            }
            _ => {
                tracing::error!("Could not parse firmware revision");
                Err(invalid_fw())
            }
        }
    }
}

impl From<InternalFirmware> for Firmware {
    fn from(fw: InternalFirmware) -> Self {
        match fw {
            InternalFirmware::Betaflight4_2_0 => Self::Betaflight(FirmwareVersion::new(4, 2, 0)),
            InternalFirmware::Betaflight4_2_6 => Self::Betaflight(FirmwareVersion::new(4, 2, 6)),
            InternalFirmware::Betaflight4_2_8 => Self::Betaflight(FirmwareVersion::new(4, 2, 8)),
            InternalFirmware::Betaflight4_2_9 => Self::Betaflight(FirmwareVersion::new(4, 2, 9)),
            InternalFirmware::Betaflight4_2_11 => Self::Betaflight(FirmwareVersion::new(4, 2, 11)),
            InternalFirmware::Betaflight4_3_0 => Self::Betaflight(FirmwareVersion::new(4, 3, 0)),
            InternalFirmware::Betaflight4_3_1 => Self::Betaflight(FirmwareVersion::new(4, 3, 1)),
            InternalFirmware::Inav3_0_1 => Self::Inav(FirmwareVersion::new(3, 0, 1)),
            InternalFirmware::Inav5_0_0 => Self::Inav(FirmwareVersion::new(5, 0, 0)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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
struct RawHeaderValue<'data, T> {
    header: &'data str,
    raw: &'data str,
    value: T,
}

impl<T> RawHeaderValue<'_, T> {
    fn invalid_header_error(&self) -> ParseError {
        ParseError::InvalidHeader {
            header: self.header.to_owned(),
            value: self.raw.to_owned(),
        }
    }
}

impl<'data, T: FromStr> RawHeaderValue<'data, T> {
    fn parse(header: &'data str, raw: &'data str) -> Result<Self, <T as FromStr>::Err> {
        Ok(Self {
            header,
            raw,
            value: raw.parse()?,
        })
    }
}

#[derive(Debug)]
struct State<'data> {
    main_frames: MainFrameDefBuilder<'data>,
    slow_frames: SlowFrameDefBuilder<'data>,
    gps_frames: GpsFrameDefBuilder<'data>,
    gps_home_frames: GpsHomeFrameDefBuilder<'data>,

    firmware_revision: Option<&'data str>,
    firmware_date: Option<&'data str>,
    firmware_kind: Option<&'data str>,
    board_info: Option<&'data str>,
    craft_name: Option<&'data str>,

    debug_mode: Option<RawHeaderValue<'data, u32>>,
    disabled_fields: u32,
    features: u32,
    pwm_protocol: Option<RawHeaderValue<'data, u32>>,

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
            main_frames: MainFrameDef::builder(),
            slow_frames: SlowFrameDef::builder(),
            gps_frames: GpsFrameDef::builder(),
            gps_home_frames: GpsHomeFrameDef::builder(),

            firmware_revision: None,
            firmware_date: None,
            firmware_kind: None,
            board_info: None,
            craft_name: None,

            debug_mode: None,
            disabled_fields: 0,
            features: 0,
            pwm_protocol: None,

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
                "Firmware revision" => self.firmware_revision = Some(value),
                "Firmware date" => self.firmware_date = Some(value),
                "Firmware type" => self.firmware_kind = Some(value),
                "Board information" => self.board_info = Some(value),
                "Craft name" => self.craft_name = Some(value),

                "debug_mode" => {
                    let debug_mode = RawHeaderValue::parse(header, value).map_err(|_| ())?;
                    self.debug_mode = Some(debug_mode);
                }
                "fields_disabled_mask" => self.disabled_fields = value.parse().map_err(|_| ())?,
                "features" => self.features = as_u32(value.parse().map_err(|_| ())?),
                "motor_pwm_protocol" => {
                    let protocol = RawHeaderValue::parse(header, value).map_err(|_| ())?;
                    self.pwm_protocol = Some(protocol);
                }

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
        let firmware = InternalFirmware::parse(firmware_revision)?;

        // TODO: log where each error comes from
        let headers = Headers {
            main_frame_def: self.main_frames.parse()?,
            slow_frame_def: self.slow_frames.parse()?,
            gps_frame_def: self.gps_frames.parse()?,
            gps_home_frame_def: self.gps_home_frames.parse()?,

            firmware_revision,
            firmware,
            firmware_date: self.firmware_date,
            board_info: self.board_info.map(str::trim).filter(not_empty),
            craft_name: self.craft_name.map(str::trim).filter(not_empty),

            debug_mode: self.debug_mode.map_or(Ok(DebugMode::None), |raw| {
                DebugMode::new(raw.value, firmware).ok_or_else(|| raw.invalid_header_error())
            })?,
            disabled_fields: DisabledFields::new(self.disabled_fields, firmware),
            features: FeatureSet::new(self.features, firmware),
            pwm_protocol: self
                .pwm_protocol
                .ok_or(ParseError::MissingHeader)
                .and_then(|raw| {
                    PwmProtocol::new(raw.value, firmware).ok_or_else(|| raw.invalid_header_error())
                })?,

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
