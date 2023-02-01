#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// All currently enabled flight modes. See [`FlagSet`][`crate::units::FlagSet`]
/// and [`FlightMode`].
#[allow(unused_qualifications)]
pub struct FlightModeSet {
    firmware: crate::headers::Firmware,
    raw: ::bitvec::array::BitArray<[u32; 1], ::bitvec::order::Lsb0>,
}
#[allow(unused_qualifications)]
impl FlightModeSet {
    pub(crate) fn new(raw: u32, firmware: crate::headers::Firmware) -> Self {
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
    const fn from_bit(bit: u32, firmware: crate::headers::Firmware) -> Option<Self> {
        use crate::headers::Firmware::{Betaflight, Inav};
        match (bit, firmware) {
            (0u32, Betaflight(_)) => Some(Self::Arm),
            (0u32, Inav(_)) => Some(Self::Angle),
            (1u32, Betaflight(_)) => Some(Self::Angle),
            (1u32, Inav(_)) => Some(Self::Horizon),
            (2u32, Betaflight(_)) => Some(Self::Horizon),
            (2u32, Inav(_)) => Some(Self::Heading),
            (3u32, Betaflight(_)) => Some(Self::Mag),
            (3u32, Inav(_)) => Some(Self::NavAltHold),
            (4u32, Betaflight(_)) => Some(Self::HeadFree),
            (4u32, Inav(_)) => Some(Self::NavRth),
            (5u32, Betaflight(_)) => Some(Self::Passthru),
            (5u32, Inav(_)) => Some(Self::NavPoshold),
            (6u32, Betaflight(_)) => Some(Self::Failsafe),
            (6u32, Inav(_)) => Some(Self::HeadFree),
            (7u32, Betaflight(_)) => Some(Self::GpsRescue),
            (7u32, Inav(_)) => Some(Self::NavLaunch),
            (8u32, Betaflight(_)) => Some(Self::Antigravity),
            (8u32, Inav(_)) => Some(Self::Manual),
            (9u32, Betaflight(_)) => Some(Self::HeadAdjust),
            (9u32, Inav(_)) => Some(Self::Failsafe),
            (10u32, Betaflight(_)) => Some(Self::CamStab),
            (10u32, Inav(_)) => Some(Self::AutoTune),
            (11u32, Betaflight(_)) => Some(Self::BeeperOn),
            (11u32, Inav(_)) => Some(Self::NavWp),
            (12u32, Betaflight(_)) => Some(Self::LedLow),
            (12u32, Inav(_)) => Some(Self::NavCourseHold),
            (13u32, Betaflight(_)) => Some(Self::Calibrate),
            (13u32, Inav(_)) => Some(Self::Flaperon),
            (14u32, Betaflight(_)) => Some(Self::Osd),
            (14u32, Inav(_)) => Some(Self::TurnAssistant),
            (15u32, Betaflight(_)) => Some(Self::Telemetry),
            (15u32, Inav(_)) => Some(Self::Turtle),
            (16u32, Betaflight(_)) => Some(Self::Servo1),
            (16u32, Inav(_)) => Some(Self::Soaring),
            (17u32, Betaflight(_)) => Some(Self::Servo2),
            (18u32, Betaflight(_)) => Some(Self::Servo3),
            (19u32, Betaflight(_)) => Some(Self::Blackbox),
            (20u32, Betaflight(_)) => Some(Self::Airmode),
            (21u32, Betaflight(_)) => Some(Self::ThreeD),
            (22u32, Betaflight(_)) => Some(Self::FpvAngleMix),
            (23u32, Betaflight(_)) => Some(Self::BlackboxErase),
            (24u32, Betaflight(_)) => Some(Self::Camera1),
            (25u32, Betaflight(_)) => Some(Self::Camera2),
            (26u32, Betaflight(_)) => Some(Self::Camera3),
            (27u32, Betaflight(_)) => Some(Self::Turtle),
            (28u32, Betaflight(_)) => Some(Self::Prearm),
            (29u32, Betaflight(_)) => Some(Self::BeepGpsCount),
            (30u32, Betaflight(_)) => Some(Self::VtxPitmode),
            (31u32, Betaflight(_)) => Some(Self::Paralyze),
            _ => None,
        }
    }

    const fn to_bit(self, firmware: crate::headers::Firmware) -> Option<u32> {
        use crate::headers::Firmware::{Betaflight, Inav};
        match (self, firmware) {
            (Self::Airmode, Betaflight(_)) => Some(20u32),
            (Self::Angle, Betaflight(_)) => Some(1u32),
            (Self::Angle, Inav(_)) => Some(0u32),
            (Self::Antigravity, Betaflight(_)) => Some(8u32),
            (Self::Arm, Betaflight(_)) => Some(0u32),
            (Self::AutoTune, Inav(_)) => Some(10u32),
            (Self::BeepGpsCount, Betaflight(_)) => Some(29u32),
            (Self::BeeperOn, Betaflight(_)) => Some(11u32),
            (Self::Blackbox, Betaflight(_)) => Some(19u32),
            (Self::BlackboxErase, Betaflight(_)) => Some(23u32),
            (Self::Calibrate, Betaflight(_)) => Some(13u32),
            (Self::CamStab, Betaflight(_)) => Some(10u32),
            (Self::Camera1, Betaflight(_)) => Some(24u32),
            (Self::Camera2, Betaflight(_)) => Some(25u32),
            (Self::Camera3, Betaflight(_)) => Some(26u32),
            (Self::Failsafe, Betaflight(_)) => Some(6u32),
            (Self::Failsafe, Inav(_)) => Some(9u32),
            (Self::Flaperon, Inav(_)) => Some(13u32),
            (Self::FpvAngleMix, Betaflight(_)) => Some(22u32),
            (Self::GpsRescue, Betaflight(_)) => Some(7u32),
            (Self::HeadAdjust, Betaflight(_)) => Some(9u32),
            (Self::HeadFree, Betaflight(_)) => Some(4u32),
            (Self::HeadFree, Inav(_)) => Some(6u32),
            (Self::Heading, Inav(_)) => Some(2u32),
            (Self::Horizon, Betaflight(_)) => Some(2u32),
            (Self::Horizon, Inav(_)) => Some(1u32),
            (Self::LedLow, Betaflight(_)) => Some(12u32),
            (Self::Mag, Betaflight(_)) => Some(3u32),
            (Self::Manual, Inav(_)) => Some(8u32),
            (Self::NavAltHold, Inav(_)) => Some(3u32),
            (Self::NavCourseHold, Inav(_)) => Some(12u32),
            (Self::NavLaunch, Inav(_)) => Some(7u32),
            (Self::NavPoshold, Inav(_)) => Some(5u32),
            (Self::NavRth, Inav(_)) => Some(4u32),
            (Self::NavWp, Inav(_)) => Some(11u32),
            (Self::Osd, Betaflight(_)) => Some(14u32),
            (Self::Paralyze, Betaflight(_)) => Some(31u32),
            (Self::Passthru, Betaflight(_)) => Some(5u32),
            (Self::Prearm, Betaflight(_)) => Some(28u32),
            (Self::Servo1, Betaflight(_)) => Some(16u32),
            (Self::Servo2, Betaflight(_)) => Some(17u32),
            (Self::Servo3, Betaflight(_)) => Some(18u32),
            (Self::Soaring, Inav(_)) => Some(16u32),
            (Self::Telemetry, Betaflight(_)) => Some(15u32),
            (Self::ThreeD, Betaflight(_)) => Some(21u32),
            (Self::TurnAssistant, Inav(_)) => Some(14u32),
            (Self::Turtle, Betaflight(_)) => Some(27u32),
            (Self::Turtle, Inav(_)) => Some(15u32),
            (Self::VtxPitmode, Betaflight(_)) => Some(30u32),
            _ => None,
        }
    }
}
