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
            .filter_map(|bit| {
                let flag = <FlightMode>::from_bit(bit as u32, self.firmware)?;
                let name = <FlightMode as crate::units::Flag>::as_name(&flag);
                Some(name)
            })
            .collect()
    }
}
impl ::core::fmt::Display for FlightModeSet {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let names = <Self as crate::units::FlagSet>::as_names(self);
        f.write_str(&names.join("|"))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A flight mode. See [`Flag`][`crate::units::Flag`].
pub enum FlightMode {
    /// `ACRO TRAINER` (Betaflight only)
    AcroTrainer,
    /// `AIRMODE`
    Airmode,
    /// `ANGLE`
    Angle,
    /// `ANTI GRAVITY` (Betaflight only)
    AntiGravity,
    /// `ARM`
    Arm,
    /// `AUTOLEVEL` (INAV only)
    AutoLevel,
    /// `AUTOTRIM` (INAV only)
    AutoTrim,
    /// `AUTOTUNE` (INAV only)
    AutoTune,
    /// `BEEP GPS COUNT` (Betaflight only)
    BeepGpsCount,
    /// `BEEPER MUTE` (Betaflight only)
    BeeperMute,
    /// `BEEPERON`
    BeeperOn,
    /// `BLACKBOX`
    Blackbox,
    /// `BLACKBOX ERASE` (Betaflight only)
    BlackboxErase,
    /// `BRAKING` (INAV only)
    Braking,
    /// `CALIB` (Betaflight only)
    Calibration,
    /// `CAMSTAB`
    CamStab,
    /// `CAMERA1`
    Camera1,
    /// `CAMERA2`
    Camera2,
    /// `CAMERA3`
    Camera3,
    /// `FAILSAFE`
    Failsafe,
    /// `FLAPERON` (INAV only)
    Flaperon,
    /// `FPV ANGLE MIX`
    FpvAngleMix,
    /// `GCSNAV` (INAV only)
    GcsNav,
    /// `GPS RESCUE` (Betaflight only)
    GpsRescue,
    /// `HEADADJ`
    HeadAdjust,
    /// `HEADFREE`
    HeadFree,
    /// `HEADINGHOLD` (INAV only)
    HeadingHold,
    /// `HOMERESET` (INAV only)
    HomeReset,
    /// `HORIZON`
    Horizon,
    /// `KILLSWITCH` (INAV only)
    Killswitch,
    /// `LAUNCH CONTROL` (Betaflight only)
    LaunchControl,
    /// `LEDLOW`
    LedLow,
    /// `LIGHTS` (INAV only)
    Lights,
    /// `LOITERDIRCHN` (INAV only)
    LoiterChange,
    /// `MAG` (Betaflight only)
    Mag,
    /// `MANUAL` (INAV only)
    Manual,
    /// `CHANGEMISSION` (INAV only)
    MissionChange,
    /// `MSPOVERRIDE` (Betaflight only)
    MspOverride,
    /// `MSPRCOVERRIDE` (INAV only)
    MspRcOverride,
    /// `NAV ALTHOLD` (INAV only)
    NavAltitudeHold,
    /// `NAVCOURSEHOLD` (INAV only)
    NavCourseHold,
    /// `NAV CRUISE` (INAV only)
    NavCruise,
    /// `NAV LAUNCH` (INAV only)
    NavLaunch,
    /// `NAV POSHOLD` (INAV only)
    NavPositionHold,
    /// `NAV RTH` (INAV only)
    NavRth,
    /// `NAV WP` (INAV only)
    NavWaypoint,
    /// `OSD`
    Osd,
    /// `OSDALT1` (INAV only)
    OsdAlt1,
    /// `OSDALT2` (INAV only)
    OsdAlt2,
    /// `OSDALT3` (INAV only)
    OsdAlt3,
    /// `PARALYZE` (Betaflight only)
    Paralyze,
    /// `PASSTHRU` (Betaflight only)
    Passthru,
    /// `PID AUDIO` (Betaflight only)
    PidAudio,
    /// `PLANWPMISSION` (INAV only)
    PlanWpMission,
    /// `PREARM`
    Prearm,
    /// `SERVO1` (Betaflight only)
    Servo1,
    /// `SERVO2` (Betaflight only)
    Servo2,
    /// `SERVO3` (Betaflight only)
    Servo3,
    /// `SOARING` (INAV only)
    Soaring,
    /// `STICK COMMAND DISABLE` (Betaflight only)
    StickCommandDisable,
    /// `SURFACE` (INAV only)
    Surface,
    /// `TELEMETRY`
    Telemetry,
    /// `3D` (Betaflight only)
    ThreeD,
    /// `TURNASSIST` (INAV only)
    TurnAssist,
    /// `TURTLE`
    Turtle,
    /// `USER1`
    User1,
    /// `USER2`
    User2,
    /// `USER3`
    User3,
    /// `USER4` (Betaflight only)
    User4,
    /// `VTX CONTROL DISABLE` (Betaflight only)
    VtxControlDisable,
    /// `VTX PIT MODE` (Betaflight only)
    VtxPitMode,
}
#[allow(unused_qualifications)]
impl crate::units::Flag for FlightMode {
    fn as_name(&self) -> &'static str {
        match self {
            Self::AcroTrainer => "ACRO TRAINER",
            Self::Airmode => "AIRMODE",
            Self::Angle => "ANGLE",
            Self::AntiGravity => "ANTI GRAVITY",
            Self::Arm => "ARM",
            Self::AutoLevel => "AUTOLEVEL",
            Self::AutoTrim => "AUTOTRIM",
            Self::AutoTune => "AUTOTUNE",
            Self::BeepGpsCount => "BEEP GPS COUNT",
            Self::BeeperMute => "BEEPER MUTE",
            Self::BeeperOn => "BEEPERON",
            Self::Blackbox => "BLACKBOX",
            Self::BlackboxErase => "BLACKBOX ERASE",
            Self::Braking => "BRAKING",
            Self::Calibration => "CALIB",
            Self::CamStab => "CAMSTAB",
            Self::Camera1 => "CAMERA1",
            Self::Camera2 => "CAMERA2",
            Self::Camera3 => "CAMERA3",
            Self::Failsafe => "FAILSAFE",
            Self::Flaperon => "FLAPERON",
            Self::FpvAngleMix => "FPV ANGLE MIX",
            Self::GcsNav => "GCSNAV",
            Self::GpsRescue => "GPS RESCUE",
            Self::HeadAdjust => "HEADADJ",
            Self::HeadFree => "HEADFREE",
            Self::HeadingHold => "HEADINGHOLD",
            Self::HomeReset => "HOMERESET",
            Self::Horizon => "HORIZON",
            Self::Killswitch => "KILLSWITCH",
            Self::LaunchControl => "LAUNCH CONTROL",
            Self::LedLow => "LEDLOW",
            Self::Lights => "LIGHTS",
            Self::LoiterChange => "LOITERDIRCHN",
            Self::Mag => "MAG",
            Self::Manual => "MANUAL",
            Self::MissionChange => "CHANGEMISSION",
            Self::MspOverride => "MSPOVERRIDE",
            Self::MspRcOverride => "MSPRCOVERRIDE",
            Self::NavAltitudeHold => "NAV ALTHOLD",
            Self::NavCourseHold => "NAVCOURSEHOLD",
            Self::NavCruise => "NAV CRUISE",
            Self::NavLaunch => "NAV LAUNCH",
            Self::NavPositionHold => "NAV POSHOLD",
            Self::NavRth => "NAV RTH",
            Self::NavWaypoint => "NAV WP",
            Self::Osd => "OSD",
            Self::OsdAlt1 => "OSDALT1",
            Self::OsdAlt2 => "OSDALT2",
            Self::OsdAlt3 => "OSDALT3",
            Self::Paralyze => "PARALYZE",
            Self::Passthru => "PASSTHRU",
            Self::PidAudio => "PID AUDIO",
            Self::PlanWpMission => "PLANWPMISSION",
            Self::Prearm => "PREARM",
            Self::Servo1 => "SERVO1",
            Self::Servo2 => "SERVO2",
            Self::Servo3 => "SERVO3",
            Self::Soaring => "SOARING",
            Self::StickCommandDisable => "STICK COMMAND DISABLE",
            Self::Surface => "SURFACE",
            Self::Telemetry => "TELEMETRY",
            Self::ThreeD => "3D",
            Self::TurnAssist => "TURNASSIST",
            Self::Turtle => "TURTLE",
            Self::User1 => "USER1",
            Self::User2 => "USER2",
            Self::User3 => "USER3",
            Self::User4 => "USER4",
            Self::VtxControlDisable => "VTX CONTROL DISABLE",
            Self::VtxPitMode => "VTX PIT MODE",
        }
    }
}
impl ::core::fmt::Display for FlightMode {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let s = <Self as crate::units::Flag>::as_name(self);
        f.write_str(s)
    }
}
#[allow(
    unused_imports,
    unused_qualifications,
    clippy::match_same_arms,
    clippy::unseparated_literal_suffix
)]
impl FlightMode {
    const fn from_bit(bit: u32, firmware: crate::headers::Firmware) -> Option<Self> {
        use crate::headers::Firmware::{Betaflight, Inav};
        match (bit, firmware) {
            (0u32, _) => Some(Self::Arm),
            (1u32, _) => Some(Self::Angle),
            (2u32, _) => Some(Self::Horizon),
            (3u32, Betaflight(_)) => Some(Self::Mag),
            (3u32, Inav(_)) => Some(Self::NavAltitudeHold),
            (4u32, Betaflight(_)) => Some(Self::HeadFree),
            (4u32, Inav(_)) => Some(Self::HeadingHold),
            (5u32, Betaflight(_)) => Some(Self::Passthru),
            (5u32, Inav(_)) => Some(Self::HeadFree),
            (6u32, Betaflight(_)) => Some(Self::Failsafe),
            (6u32, Inav(_)) => Some(Self::HeadAdjust),
            (7u32, Betaflight(_)) => Some(Self::GpsRescue),
            (7u32, Inav(_)) => Some(Self::CamStab),
            (8u32, Betaflight(_)) => Some(Self::AntiGravity),
            (8u32, Inav(_)) => Some(Self::NavRth),
            (9u32, Betaflight(_)) => Some(Self::HeadAdjust),
            (9u32, Inav(_)) => Some(Self::NavPositionHold),
            (10u32, Betaflight(_)) => Some(Self::CamStab),
            (10u32, Inav(_)) => Some(Self::Manual),
            (11u32, _) => Some(Self::BeeperOn),
            (12u32, _) => Some(Self::LedLow),
            (13u32, Betaflight(_)) => Some(Self::Calibration),
            (13u32, Inav(_)) => Some(Self::Lights),
            (14u32, Betaflight(_)) => Some(Self::Osd),
            (14u32, Inav(_)) => Some(Self::NavLaunch),
            (15u32, Betaflight(_)) => Some(Self::Telemetry),
            (15u32, Inav(_)) => Some(Self::Osd),
            (16u32, Betaflight(_)) => Some(Self::Servo1),
            (16u32, Inav(_)) => Some(Self::Telemetry),
            (17u32, Betaflight(_)) => Some(Self::Servo2),
            (17u32, Inav(_)) => Some(Self::Blackbox),
            (18u32, Betaflight(_)) => Some(Self::Servo3),
            (18u32, Inav(_)) => Some(Self::Failsafe),
            (19u32, Betaflight(_)) => Some(Self::Blackbox),
            (19u32, Inav(_)) => Some(Self::NavWaypoint),
            (20u32, _) => Some(Self::Airmode),
            (21u32, Betaflight(_)) => Some(Self::ThreeD),
            (21u32, Inav(_)) => Some(Self::HomeReset),
            (22u32, Betaflight(_)) => Some(Self::FpvAngleMix),
            (22u32, Inav(_)) => Some(Self::GcsNav),
            (23u32, Betaflight(_)) => Some(Self::BlackboxErase),
            (23u32, Inav(_)) => Some(Self::Killswitch),
            (24u32, Betaflight(_)) => Some(Self::Camera1),
            (24u32, Inav(_)) => Some(Self::Surface),
            (25u32, Betaflight(_)) => Some(Self::Camera2),
            (25u32, Inav(_)) => Some(Self::Flaperon),
            (26u32, Betaflight(_)) => Some(Self::Camera3),
            (26u32, Inav(_)) => Some(Self::TurnAssist),
            (27u32, Betaflight(_)) => Some(Self::Turtle),
            (27u32, Inav(_)) => Some(Self::AutoTrim),
            (28u32, Betaflight(_)) => Some(Self::Prearm),
            (28u32, Inav(_)) => Some(Self::AutoTune),
            (29u32, Betaflight(_)) => Some(Self::BeepGpsCount),
            (29u32, Inav(_)) => Some(Self::Camera1),
            (30u32, Betaflight(_)) => Some(Self::VtxPitMode),
            (30u32, Inav(_)) => Some(Self::Camera2),
            (31u32, Betaflight(_)) => Some(Self::Paralyze),
            (31u32, Inav(_)) => Some(Self::Camera3),
            (32u32, Betaflight(_)) => Some(Self::User1),
            (32u32, Inav(_)) => Some(Self::OsdAlt1),
            (33u32, Betaflight(_)) => Some(Self::User2),
            (33u32, Inav(_)) => Some(Self::OsdAlt2),
            (34u32, Betaflight(_)) => Some(Self::User3),
            (34u32, Inav(_)) => Some(Self::OsdAlt3),
            (35u32, Betaflight(_)) => Some(Self::User4),
            (35u32, Inav(_)) => Some(Self::NavCourseHold),
            (36u32, Betaflight(_)) => Some(Self::PidAudio),
            (36u32, Inav(_)) => Some(Self::Braking),
            (37u32, Betaflight(_)) => Some(Self::AcroTrainer),
            (37u32, Inav(_)) => Some(Self::User1),
            (38u32, Betaflight(_)) => Some(Self::VtxControlDisable),
            (38u32, Inav(_)) => Some(Self::User2),
            (39u32, Betaflight(_)) => Some(Self::LaunchControl),
            (39u32, Inav(_)) => Some(Self::FpvAngleMix),
            (40u32, Betaflight(_)) => Some(Self::MspOverride),
            (40u32, Inav(_)) => Some(Self::LoiterChange),
            (41u32, Betaflight(_)) => Some(Self::StickCommandDisable),
            (41u32, Inav(_)) => Some(Self::MspRcOverride),
            (42u32, Betaflight(_)) => Some(Self::BeeperMute),
            (42u32, Inav(_)) => Some(Self::Prearm),
            (43u32, Inav(_)) => Some(Self::Turtle),
            (44u32, Inav(_)) => Some(Self::NavCruise),
            (45u32, Inav(_)) => Some(Self::AutoLevel),
            (46u32, Inav(_)) => Some(Self::PlanWpMission),
            (47u32, Inav(_)) => Some(Self::Soaring),
            (48u32, Inav(_)) => Some(Self::User3),
            (49u32, Inav(_)) => Some(Self::MissionChange),
            _ => None,
        }
    }

    const fn to_bit(self, firmware: crate::headers::Firmware) -> Option<u32> {
        use crate::headers::Firmware::{Betaflight, Inav};
        match (self, firmware) {
            (Self::AcroTrainer, Betaflight(_)) => Some(37u32),
            (Self::Airmode, _) => Some(20u32),
            (Self::Angle, _) => Some(1u32),
            (Self::AntiGravity, Betaflight(_)) => Some(8u32),
            (Self::Arm, _) => Some(0u32),
            (Self::AutoLevel, Inav(_)) => Some(45u32),
            (Self::AutoTrim, Inav(_)) => Some(27u32),
            (Self::AutoTune, Inav(_)) => Some(28u32),
            (Self::BeepGpsCount, Betaflight(_)) => Some(29u32),
            (Self::BeeperMute, Betaflight(_)) => Some(42u32),
            (Self::BeeperOn, _) => Some(11u32),
            (Self::Blackbox, Betaflight(_)) => Some(19u32),
            (Self::Blackbox, Inav(_)) => Some(17u32),
            (Self::BlackboxErase, Betaflight(_)) => Some(23u32),
            (Self::Braking, Inav(_)) => Some(36u32),
            (Self::Calibration, Betaflight(_)) => Some(13u32),
            (Self::CamStab, Betaflight(_)) => Some(10u32),
            (Self::CamStab, Inav(_)) => Some(7u32),
            (Self::Camera1, Betaflight(_)) => Some(24u32),
            (Self::Camera1, Inav(_)) => Some(29u32),
            (Self::Camera2, Betaflight(_)) => Some(25u32),
            (Self::Camera2, Inav(_)) => Some(30u32),
            (Self::Camera3, Betaflight(_)) => Some(26u32),
            (Self::Camera3, Inav(_)) => Some(31u32),
            (Self::Failsafe, Betaflight(_)) => Some(6u32),
            (Self::Failsafe, Inav(_)) => Some(18u32),
            (Self::Flaperon, Inav(_)) => Some(25u32),
            (Self::FpvAngleMix, Betaflight(_)) => Some(22u32),
            (Self::FpvAngleMix, Inav(_)) => Some(39u32),
            (Self::GcsNav, Inav(_)) => Some(22u32),
            (Self::GpsRescue, Betaflight(_)) => Some(7u32),
            (Self::HeadAdjust, Betaflight(_)) => Some(9u32),
            (Self::HeadAdjust, Inav(_)) => Some(6u32),
            (Self::HeadFree, Betaflight(_)) => Some(4u32),
            (Self::HeadFree, Inav(_)) => Some(5u32),
            (Self::HeadingHold, Inav(_)) => Some(4u32),
            (Self::HomeReset, Inav(_)) => Some(21u32),
            (Self::Horizon, _) => Some(2u32),
            (Self::Killswitch, Inav(_)) => Some(23u32),
            (Self::LaunchControl, Betaflight(_)) => Some(39u32),
            (Self::LedLow, _) => Some(12u32),
            (Self::Lights, Inav(_)) => Some(13u32),
            (Self::LoiterChange, Inav(_)) => Some(40u32),
            (Self::Mag, Betaflight(_)) => Some(3u32),
            (Self::Manual, Inav(_)) => Some(10u32),
            (Self::MissionChange, Inav(_)) => Some(49u32),
            (Self::MspOverride, Betaflight(_)) => Some(40u32),
            (Self::MspRcOverride, Inav(_)) => Some(41u32),
            (Self::NavAltitudeHold, Inav(_)) => Some(3u32),
            (Self::NavCourseHold, Inav(_)) => Some(35u32),
            (Self::NavCruise, Inav(_)) => Some(44u32),
            (Self::NavLaunch, Inav(_)) => Some(14u32),
            (Self::NavPositionHold, Inav(_)) => Some(9u32),
            (Self::NavRth, Inav(_)) => Some(8u32),
            (Self::NavWaypoint, Inav(_)) => Some(19u32),
            (Self::Osd, Betaflight(_)) => Some(14u32),
            (Self::Osd, Inav(_)) => Some(15u32),
            (Self::OsdAlt1, Inav(_)) => Some(32u32),
            (Self::OsdAlt2, Inav(_)) => Some(33u32),
            (Self::OsdAlt3, Inav(_)) => Some(34u32),
            (Self::Paralyze, Betaflight(_)) => Some(31u32),
            (Self::Passthru, Betaflight(_)) => Some(5u32),
            (Self::PidAudio, Betaflight(_)) => Some(36u32),
            (Self::PlanWpMission, Inav(_)) => Some(46u32),
            (Self::Prearm, Betaflight(_)) => Some(28u32),
            (Self::Prearm, Inav(_)) => Some(42u32),
            (Self::Servo1, Betaflight(_)) => Some(16u32),
            (Self::Servo2, Betaflight(_)) => Some(17u32),
            (Self::Servo3, Betaflight(_)) => Some(18u32),
            (Self::Soaring, Inav(_)) => Some(47u32),
            (Self::StickCommandDisable, Betaflight(_)) => Some(41u32),
            (Self::Surface, Inav(_)) => Some(24u32),
            (Self::Telemetry, Betaflight(_)) => Some(15u32),
            (Self::Telemetry, Inav(_)) => Some(16u32),
            (Self::ThreeD, Betaflight(_)) => Some(21u32),
            (Self::TurnAssist, Inav(_)) => Some(26u32),
            (Self::Turtle, Betaflight(_)) => Some(27u32),
            (Self::Turtle, Inav(_)) => Some(43u32),
            (Self::User1, Betaflight(_)) => Some(32u32),
            (Self::User1, Inav(_)) => Some(37u32),
            (Self::User2, Betaflight(_)) => Some(33u32),
            (Self::User2, Inav(_)) => Some(38u32),
            (Self::User3, Betaflight(_)) => Some(34u32),
            (Self::User3, Inav(_)) => Some(48u32),
            (Self::User4, Betaflight(_)) => Some(35u32),
            (Self::VtxControlDisable, Betaflight(_)) => Some(38u32),
            (Self::VtxPitMode, Betaflight(_)) => Some(30u32),
            _ => None,
        }
    }
}
