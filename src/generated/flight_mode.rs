#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// All currently enabled flight modes. See [`FlagSet`][`crate::units::FlagSet`]
/// and [`FlightMode`].
#[allow(unused_qualifications)]
pub struct FlightModeSet {
    firmware: crate::headers::FirmwareKind,
    raw: ::bitvec::array::BitArray<[u32; 1], ::bitvec::order::Lsb0>,
}
#[allow(unused_qualifications)]
impl FlightModeSet {
    pub(crate) fn new(raw: u32, firmware: crate::headers::FirmwareKind) -> Self {
        Self {
            firmware,
            raw: ::bitvec::array::BitArray::new([raw]),
        }
    }
}
#[allow(unused_qualifications, clippy::cast_possible_truncation)]
impl crate::units::FlagSet for FlightModeSet {
    type Flag = FlightMode;

    fn is_set(&self, flag: Self::Flag) -> bool {
        flag.to_bit(self.firmware)
            .map_or(false, |bit| self.raw[bit as usize])
    }

    fn as_names(&self) -> ::alloc::vec::Vec<&'static str> {
        self.raw
            .iter_ones()
            .filter_map(|bit| Some(<FlightMode>::from_bit(bit as u32, self.firmware)?.as_name()))
            .collect()
    }
}
impl ::core::fmt::Display for FlightModeSet {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.write_str(&self.as_names().join("|"))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A flight mode. See [`Flag`][`crate::units::Flag`].
pub enum FlightMode {
    /// `AIRMODE` (Betaflight only)
    Airmode,
    /// `ANGLE`
    Angle,
    /// `ANTIGRAVITY` (Betaflight only)
    Antigravity,
    /// `ARM` (Betaflight only)
    Arm,
    /// `AUTO_TUNE` (INAV only)
    AutoTune,
    /// `BEEP_GPS_COUNT` (Betaflight only)
    BeepGpsCount,
    /// `BEEPER_ON` (Betaflight only)
    BeeperOn,
    /// `BLACKBOX` (Betaflight only)
    Blackbox,
    /// `BLACKBOX_ERASE` (Betaflight only)
    BlackboxErase,
    /// `CALIB` (Betaflight only)
    Calibrate,
    /// `CAM_STAB` (Betaflight only)
    CamStab,
    /// `CAMERA1` (Betaflight only)
    Camera1,
    /// `CAMERA2` (Betaflight only)
    Camera2,
    /// `CAMERA3` (Betaflight only)
    Camera3,
    /// `FAILSAFE`
    Failsafe,
    /// `FLAPERON` (INAV only)
    Flaperon,
    /// `FPV_ANGLE_MIX` (Betaflight only)
    FpvAngleMix,
    /// `GPS_RESCUE` (Betaflight only)
    GpsRescue,
    /// `HEAD_ADJUST` (Betaflight only)
    HeadAdjust,
    /// `HEAD_FREE`
    HeadFree,
    /// `HEADING` (INAV only)
    Heading,
    /// `HORIZON`
    Horizon,
    /// `LED_LOW` (Betaflight only)
    LedLow,
    /// `MAG` (Betaflight only)
    Mag,
    /// `MANUAL` (INAV only)
    Manual,
    /// `NAV_ALT_HOLD` (INAV only)
    NavAltHold,
    /// `NAV_COURSE_HOLD` (INAV only)
    NavCourseHold,
    /// `NAV_LAUNCH` (INAV only)
    NavLaunch,
    /// `NAV_POSHOLD` (INAV only)
    NavPoshold,
    /// `NAV_RTH` (INAV only)
    NavRth,
    /// `NAV_WP` (INAV only)
    NavWp,
    /// `OSD` (Betaflight only)
    Osd,
    /// `PARALYZE` (Betaflight only)
    Paralyze,
    /// `PASSTHRU` (Betaflight only)
    Passthru,
    /// `PREARM` (Betaflight only)
    Prearm,
    /// `SERVO1` (Betaflight only)
    Servo1,
    /// `SERVO2` (Betaflight only)
    Servo2,
    /// `SERVO3` (Betaflight only)
    Servo3,
    /// `SOARING` (INAV only)
    Soaring,
    /// `TELEMETRY` (Betaflight only)
    Telemetry,
    /// `3D` (Betaflight only)
    ThreeD,
    /// `TURN_ASSISTANT` (INAV only)
    TurnAssistant,
    /// `TURTLE`
    Turtle,
    /// `VTX_PITMODE` (Betaflight only)
    VtxPitmode,
}
#[allow(unused_qualifications)]
impl crate::units::Flag for FlightMode {
    fn as_name(&self) -> &'static str {
        match self {
            Self::Airmode => "AIRMODE",
            Self::Angle => "ANGLE",
            Self::Antigravity => "ANTIGRAVITY",
            Self::Arm => "ARM",
            Self::AutoTune => "AUTO_TUNE",
            Self::BeepGpsCount => "BEEP_GPS_COUNT",
            Self::BeeperOn => "BEEPER_ON",
            Self::Blackbox => "BLACKBOX",
            Self::BlackboxErase => "BLACKBOX_ERASE",
            Self::Calibrate => "CALIB",
            Self::CamStab => "CAM_STAB",
            Self::Camera1 => "CAMERA1",
            Self::Camera2 => "CAMERA2",
            Self::Camera3 => "CAMERA3",
            Self::Failsafe => "FAILSAFE",
            Self::Flaperon => "FLAPERON",
            Self::FpvAngleMix => "FPV_ANGLE_MIX",
            Self::GpsRescue => "GPS_RESCUE",
            Self::HeadAdjust => "HEAD_ADJUST",
            Self::HeadFree => "HEAD_FREE",
            Self::Heading => "HEADING",
            Self::Horizon => "HORIZON",
            Self::LedLow => "LED_LOW",
            Self::Mag => "MAG",
            Self::Manual => "MANUAL",
            Self::NavAltHold => "NAV_ALT_HOLD",
            Self::NavCourseHold => "NAV_COURSE_HOLD",
            Self::NavLaunch => "NAV_LAUNCH",
            Self::NavPoshold => "NAV_POSHOLD",
            Self::NavRth => "NAV_RTH",
            Self::NavWp => "NAV_WP",
            Self::Osd => "OSD",
            Self::Paralyze => "PARALYZE",
            Self::Passthru => "PASSTHRU",
            Self::Prearm => "PREARM",
            Self::Servo1 => "SERVO1",
            Self::Servo2 => "SERVO2",
            Self::Servo3 => "SERVO3",
            Self::Soaring => "SOARING",
            Self::Telemetry => "TELEMETRY",
            Self::ThreeD => "3D",
            Self::TurnAssistant => "TURN_ASSISTANT",
            Self::Turtle => "TURTLE",
            Self::VtxPitmode => "VTX_PITMODE",
        }
    }
}
impl ::core::fmt::Display for FlightMode {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.write_str(self.as_name())
    }
}
#[allow(
    unused_qualifications,
    clippy::match_same_arms,
    clippy::unseparated_literal_suffix
)]
impl FlightMode {
    const fn from_bit(bit: u32, firmware: crate::headers::FirmwareKind) -> Option<Self> {
        use crate::headers::FirmwareKind::{Betaflight, Inav};
        match (bit, firmware) {
            (0u32, Betaflight) => Some(Self::Arm),
            (0u32, Inav) => Some(Self::Angle),
            (1u32, Betaflight) => Some(Self::Angle),
            (1u32, Inav) => Some(Self::Horizon),
            (2u32, Betaflight) => Some(Self::Horizon),
            (2u32, Inav) => Some(Self::Heading),
            (3u32, Betaflight) => Some(Self::Mag),
            (3u32, Inav) => Some(Self::NavAltHold),
            (4u32, Betaflight) => Some(Self::HeadFree),
            (4u32, Inav) => Some(Self::NavRth),
            (5u32, Betaflight) => Some(Self::Passthru),
            (5u32, Inav) => Some(Self::NavPoshold),
            (6u32, Betaflight) => Some(Self::Failsafe),
            (6u32, Inav) => Some(Self::HeadFree),
            (7u32, Betaflight) => Some(Self::GpsRescue),
            (7u32, Inav) => Some(Self::NavLaunch),
            (8u32, Betaflight) => Some(Self::Antigravity),
            (8u32, Inav) => Some(Self::Manual),
            (9u32, Betaflight) => Some(Self::HeadAdjust),
            (9u32, Inav) => Some(Self::Failsafe),
            (10u32, Betaflight) => Some(Self::CamStab),
            (10u32, Inav) => Some(Self::AutoTune),
            (11u32, Betaflight) => Some(Self::BeeperOn),
            (11u32, Inav) => Some(Self::NavWp),
            (12u32, Betaflight) => Some(Self::LedLow),
            (12u32, Inav) => Some(Self::NavCourseHold),
            (13u32, Betaflight) => Some(Self::Calibrate),
            (13u32, Inav) => Some(Self::Flaperon),
            (14u32, Betaflight) => Some(Self::Osd),
            (14u32, Inav) => Some(Self::TurnAssistant),
            (15u32, Betaflight) => Some(Self::Telemetry),
            (15u32, Inav) => Some(Self::Turtle),
            (16u32, Betaflight) => Some(Self::Servo1),
            (16u32, Inav) => Some(Self::Soaring),
            (17u32, Betaflight) => Some(Self::Servo2),
            (18u32, Betaflight) => Some(Self::Servo3),
            (19u32, Betaflight) => Some(Self::Blackbox),
            (20u32, Betaflight) => Some(Self::Airmode),
            (21u32, Betaflight) => Some(Self::ThreeD),
            (22u32, Betaflight) => Some(Self::FpvAngleMix),
            (23u32, Betaflight) => Some(Self::BlackboxErase),
            (24u32, Betaflight) => Some(Self::Camera1),
            (25u32, Betaflight) => Some(Self::Camera2),
            (26u32, Betaflight) => Some(Self::Camera3),
            (27u32, Betaflight) => Some(Self::Turtle),
            (28u32, Betaflight) => Some(Self::Prearm),
            (29u32, Betaflight) => Some(Self::BeepGpsCount),
            (30u32, Betaflight) => Some(Self::VtxPitmode),
            (31u32, Betaflight) => Some(Self::Paralyze),
            _ => None,
        }
    }

    const fn to_bit(self, firmware: crate::headers::FirmwareKind) -> Option<u32> {
        use crate::headers::FirmwareKind::{Betaflight, Inav};
        match (self, firmware) {
            (Self::Airmode, Betaflight) => Some(20u32),
            (Self::Angle, Betaflight) => Some(1u32),
            (Self::Angle, Inav) => Some(0u32),
            (Self::Antigravity, Betaflight) => Some(8u32),
            (Self::Arm, Betaflight) => Some(0u32),
            (Self::AutoTune, Inav) => Some(10u32),
            (Self::BeepGpsCount, Betaflight) => Some(29u32),
            (Self::BeeperOn, Betaflight) => Some(11u32),
            (Self::Blackbox, Betaflight) => Some(19u32),
            (Self::BlackboxErase, Betaflight) => Some(23u32),
            (Self::Calibrate, Betaflight) => Some(13u32),
            (Self::CamStab, Betaflight) => Some(10u32),
            (Self::Camera1, Betaflight) => Some(24u32),
            (Self::Camera2, Betaflight) => Some(25u32),
            (Self::Camera3, Betaflight) => Some(26u32),
            (Self::Failsafe, Betaflight) => Some(6u32),
            (Self::Failsafe, Inav) => Some(9u32),
            (Self::Flaperon, Inav) => Some(13u32),
            (Self::FpvAngleMix, Betaflight) => Some(22u32),
            (Self::GpsRescue, Betaflight) => Some(7u32),
            (Self::HeadAdjust, Betaflight) => Some(9u32),
            (Self::HeadFree, Betaflight) => Some(4u32),
            (Self::HeadFree, Inav) => Some(6u32),
            (Self::Heading, Inav) => Some(2u32),
            (Self::Horizon, Betaflight) => Some(2u32),
            (Self::Horizon, Inav) => Some(1u32),
            (Self::LedLow, Betaflight) => Some(12u32),
            (Self::Mag, Betaflight) => Some(3u32),
            (Self::Manual, Inav) => Some(8u32),
            (Self::NavAltHold, Inav) => Some(3u32),
            (Self::NavCourseHold, Inav) => Some(12u32),
            (Self::NavLaunch, Inav) => Some(7u32),
            (Self::NavPoshold, Inav) => Some(5u32),
            (Self::NavRth, Inav) => Some(4u32),
            (Self::NavWp, Inav) => Some(11u32),
            (Self::Osd, Betaflight) => Some(14u32),
            (Self::Paralyze, Betaflight) => Some(31u32),
            (Self::Passthru, Betaflight) => Some(5u32),
            (Self::Prearm, Betaflight) => Some(28u32),
            (Self::Servo1, Betaflight) => Some(16u32),
            (Self::Servo2, Betaflight) => Some(17u32),
            (Self::Servo3, Betaflight) => Some(18u32),
            (Self::Soaring, Inav) => Some(16u32),
            (Self::Telemetry, Betaflight) => Some(15u32),
            (Self::ThreeD, Betaflight) => Some(21u32),
            (Self::TurnAssistant, Inav) => Some(14u32),
            (Self::Turtle, Betaflight) => Some(27u32),
            (Self::Turtle, Inav) => Some(15u32),
            (Self::VtxPitmode, Betaflight) => Some(30u32),
            _ => None,
        }
    }
}
