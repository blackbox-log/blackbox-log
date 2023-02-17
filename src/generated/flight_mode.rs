#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// All currently enabled flight modes. See [`FlagSet`][crate::units::FlagSet]
/// and [`FlightMode`].
#[allow(unused_qualifications)]
pub struct FlightModeSet {
    firmware: crate::headers::InternalFirmware,
    raw: ::bitvec::array::BitArray<u32, ::bitvec::order::Lsb0>,
}
#[allow(unused_qualifications, clippy::cast_possible_truncation)]
impl FlightModeSet {
    pub(crate) fn new(raw: u32, firmware: crate::headers::InternalFirmware) -> Self {
        Self {
            firmware,
            raw: ::bitvec::array::BitArray::new(raw),
        }
    }

    fn iter(&self) -> impl Iterator<Item = <Self as crate::units::FlagSet>::Flag> + '_ {
        self.raw
            .iter_ones()
            .filter_map(|bit| <FlightMode>::from_bit(bit as u32, self.firmware))
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
        self.iter()
            .map(|flag| <FlightMode as crate::units::Flag>::as_name(&flag))
            .collect()
    }
}
impl ::core::fmt::Display for FlightModeSet {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let names = <Self as crate::units::FlagSet>::as_names(self);
        f.write_str(&names.join("|"))
    }
}
#[cfg(feature = "_serde")]
#[allow(clippy::cast_possible_truncation)]
impl ::serde::Serialize for FlightModeSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(None)?;
        for flag in self.iter() {
            seq.serialize_element(&flag)?;
        }
        seq.end()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "_serde", derive(serde::Serialize))]
/// A flight mode. See [`Flag`][crate::units::Flag].
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
    unused_qualifications,
    clippy::match_same_arms,
    clippy::unseparated_literal_suffix,
    clippy::wildcard_enum_match_arm
)]
impl FlightMode {
    const fn from_bit(bit: u32, fw: crate::headers::InternalFirmware) -> Option<Self> {
        match bit {
            0u32 => Some(Self::Arm),
            1u32 => Some(Self::Angle),
            2u32 => Some(Self::Horizon),
            3u32 if fw.is_betaflight() => Some(Self::Mag),
            3u32 if fw.is_inav() => Some(Self::NavAltitudeHold),
            4u32 if fw.is_betaflight() => Some(Self::HeadFree),
            4u32 if fw.is_inav() => Some(Self::HeadingHold),
            5u32 if fw.is_betaflight() => Some(Self::Passthru),
            5u32 if fw.is_inav() => Some(Self::HeadFree),
            6u32 if fw.is_betaflight() => Some(Self::Failsafe),
            6u32 if fw.is_inav() => Some(Self::HeadAdjust),
            7u32 if fw.is_betaflight() => Some(Self::GpsRescue),
            7u32 if fw.is_inav() => Some(Self::CamStab),
            8u32 if fw.is_betaflight() => Some(Self::AntiGravity),
            8u32 if fw.is_inav() => Some(Self::NavRth),
            9u32 if fw.is_betaflight() => Some(Self::HeadAdjust),
            9u32 if fw.is_inav() => Some(Self::NavPositionHold),
            10u32 if fw.is_betaflight() => Some(Self::CamStab),
            10u32 if fw.is_inav() => Some(Self::Manual),
            11u32 => Some(Self::BeeperOn),
            12u32 => Some(Self::LedLow),
            13u32 if fw.is_betaflight() => Some(Self::Calibration),
            13u32 if fw.is_inav() => Some(Self::Lights),
            14u32 if fw.is_betaflight() => Some(Self::Osd),
            14u32 if fw.is_inav() => Some(Self::NavLaunch),
            15u32 if fw.is_betaflight() => Some(Self::Telemetry),
            15u32 if fw.is_inav() => Some(Self::Osd),
            16u32 if fw.is_betaflight() => Some(Self::Servo1),
            16u32 if fw.is_inav() => Some(Self::Telemetry),
            17u32 if fw.is_betaflight() => Some(Self::Servo2),
            17u32 if fw.is_inav() => Some(Self::Blackbox),
            18u32 if fw.is_betaflight() => Some(Self::Servo3),
            18u32 if fw.is_inav() => Some(Self::Failsafe),
            19u32 if fw.is_betaflight() => Some(Self::Blackbox),
            19u32 if fw.is_inav() => Some(Self::NavWaypoint),
            20u32 => Some(Self::Airmode),
            21u32 if fw.is_betaflight() => Some(Self::ThreeD),
            21u32 if fw.is_inav() => Some(Self::HomeReset),
            22u32 if fw.is_betaflight() => Some(Self::FpvAngleMix),
            22u32 if fw.is_inav() => Some(Self::GcsNav),
            23u32 if fw.is_betaflight() => Some(Self::BlackboxErase),
            23u32 if fw.is_inav() => Some(Self::Killswitch),
            24u32 if fw.is_betaflight() => Some(Self::Camera1),
            24u32 if fw.is_inav() => Some(Self::Surface),
            25u32 if fw.is_betaflight() => Some(Self::Camera2),
            25u32 if fw.is_inav() => Some(Self::Flaperon),
            26u32 if fw.is_betaflight() => Some(Self::Camera3),
            26u32 if fw.is_inav() => Some(Self::TurnAssist),
            27u32 if fw.is_betaflight() => Some(Self::Turtle),
            27u32 if fw.is_inav() => Some(Self::AutoTrim),
            28u32 if fw.is_betaflight() => Some(Self::Prearm),
            28u32 if fw.is_inav() => Some(Self::AutoTune),
            29u32 if fw.is_betaflight() => Some(Self::BeepGpsCount),
            29u32 if fw.is_inav() => Some(Self::Camera1),
            30u32 if fw.is_betaflight() => Some(Self::VtxPitMode),
            30u32 if fw.is_inav() => Some(Self::Camera2),
            31u32 if fw.is_betaflight() => Some(Self::Paralyze),
            31u32 if fw.is_inav() => Some(Self::Camera3),
            32u32 if fw.is_betaflight() => Some(Self::User1),
            32u32 if fw.is_inav() => Some(Self::OsdAlt1),
            33u32 if fw.is_betaflight() => Some(Self::User2),
            33u32 if fw.is_inav() => Some(Self::OsdAlt2),
            34u32 if fw.is_betaflight() => Some(Self::User3),
            34u32 if fw.is_inav() => Some(Self::OsdAlt3),
            35u32 if fw.is_betaflight() => Some(Self::User4),
            35u32 if fw.is_inav() => Some(Self::NavCourseHold),
            36u32 if fw.is_betaflight() => Some(Self::PidAudio),
            36u32 if fw.is_inav() => Some(Self::Braking),
            37u32 if fw.is_betaflight() => Some(Self::AcroTrainer),
            37u32 if fw.is_inav() => Some(Self::User1),
            38u32 if fw.is_betaflight() => Some(Self::VtxControlDisable),
            38u32 if fw.is_inav() => Some(Self::User2),
            39u32 if fw.is_betaflight() => Some(Self::LaunchControl),
            39u32 if fw.is_inav() => Some(Self::FpvAngleMix),
            40u32 if fw.is_betaflight() => Some(Self::MspOverride),
            40u32 if fw.is_inav() => Some(Self::LoiterChange),
            41u32 if fw.is_betaflight() => Some(Self::StickCommandDisable),
            41u32 if fw.is_inav() => Some(Self::MspRcOverride),
            42u32 if fw.is_betaflight() => Some(Self::BeeperMute),
            42u32 if fw.is_inav() => Some(Self::Prearm),
            43u32 if fw.is_inav() => Some(Self::Turtle),
            44u32 if fw.is_inav() => Some(Self::NavCruise),
            45u32 if fw.is_inav() => Some(Self::AutoLevel),
            46u32 if fw.is_inav() => Some(Self::PlanWpMission),
            47u32 if fw.is_inav() => Some(Self::Soaring),
            48u32 if fw.is_inav() => Some(Self::User3),
            49u32 if fw.is_inav() => Some(Self::MissionChange),
            _ => None,
        }
    }

    const fn to_bit(self, fw: crate::headers::InternalFirmware) -> Option<u32> {
        match self {
            Self::AcroTrainer if fw.is_betaflight() => Some(37u32),
            Self::Airmode => Some(20u32),
            Self::Angle => Some(1u32),
            Self::AntiGravity if fw.is_betaflight() => Some(8u32),
            Self::Arm => Some(0u32),
            Self::AutoLevel if fw.is_inav() => Some(45u32),
            Self::AutoTrim if fw.is_inav() => Some(27u32),
            Self::AutoTune if fw.is_inav() => Some(28u32),
            Self::BeepGpsCount if fw.is_betaflight() => Some(29u32),
            Self::BeeperMute if fw.is_betaflight() => Some(42u32),
            Self::BeeperOn => Some(11u32),
            Self::Blackbox if fw.is_betaflight() => Some(19u32),
            Self::Blackbox if fw.is_inav() => Some(17u32),
            Self::BlackboxErase if fw.is_betaflight() => Some(23u32),
            Self::Braking if fw.is_inav() => Some(36u32),
            Self::Calibration if fw.is_betaflight() => Some(13u32),
            Self::CamStab if fw.is_betaflight() => Some(10u32),
            Self::CamStab if fw.is_inav() => Some(7u32),
            Self::Camera1 if fw.is_betaflight() => Some(24u32),
            Self::Camera1 if fw.is_inav() => Some(29u32),
            Self::Camera2 if fw.is_betaflight() => Some(25u32),
            Self::Camera2 if fw.is_inav() => Some(30u32),
            Self::Camera3 if fw.is_betaflight() => Some(26u32),
            Self::Camera3 if fw.is_inav() => Some(31u32),
            Self::Failsafe if fw.is_betaflight() => Some(6u32),
            Self::Failsafe if fw.is_inav() => Some(18u32),
            Self::Flaperon if fw.is_inav() => Some(25u32),
            Self::FpvAngleMix if fw.is_betaflight() => Some(22u32),
            Self::FpvAngleMix if fw.is_inav() => Some(39u32),
            Self::GcsNav if fw.is_inav() => Some(22u32),
            Self::GpsRescue if fw.is_betaflight() => Some(7u32),
            Self::HeadAdjust if fw.is_betaflight() => Some(9u32),
            Self::HeadAdjust if fw.is_inav() => Some(6u32),
            Self::HeadFree if fw.is_betaflight() => Some(4u32),
            Self::HeadFree if fw.is_inav() => Some(5u32),
            Self::HeadingHold if fw.is_inav() => Some(4u32),
            Self::HomeReset if fw.is_inav() => Some(21u32),
            Self::Horizon => Some(2u32),
            Self::Killswitch if fw.is_inav() => Some(23u32),
            Self::LaunchControl if fw.is_betaflight() => Some(39u32),
            Self::LedLow => Some(12u32),
            Self::Lights if fw.is_inav() => Some(13u32),
            Self::LoiterChange if fw.is_inav() => Some(40u32),
            Self::Mag if fw.is_betaflight() => Some(3u32),
            Self::Manual if fw.is_inav() => Some(10u32),
            Self::MissionChange if fw.is_inav() => Some(49u32),
            Self::MspOverride if fw.is_betaflight() => Some(40u32),
            Self::MspRcOverride if fw.is_inav() => Some(41u32),
            Self::NavAltitudeHold if fw.is_inav() => Some(3u32),
            Self::NavCourseHold if fw.is_inav() => Some(35u32),
            Self::NavCruise if fw.is_inav() => Some(44u32),
            Self::NavLaunch if fw.is_inav() => Some(14u32),
            Self::NavPositionHold if fw.is_inav() => Some(9u32),
            Self::NavRth if fw.is_inav() => Some(8u32),
            Self::NavWaypoint if fw.is_inav() => Some(19u32),
            Self::Osd if fw.is_betaflight() => Some(14u32),
            Self::Osd if fw.is_inav() => Some(15u32),
            Self::OsdAlt1 if fw.is_inav() => Some(32u32),
            Self::OsdAlt2 if fw.is_inav() => Some(33u32),
            Self::OsdAlt3 if fw.is_inav() => Some(34u32),
            Self::Paralyze if fw.is_betaflight() => Some(31u32),
            Self::Passthru if fw.is_betaflight() => Some(5u32),
            Self::PidAudio if fw.is_betaflight() => Some(36u32),
            Self::PlanWpMission if fw.is_inav() => Some(46u32),
            Self::Prearm if fw.is_betaflight() => Some(28u32),
            Self::Prearm if fw.is_inav() => Some(42u32),
            Self::Servo1 if fw.is_betaflight() => Some(16u32),
            Self::Servo2 if fw.is_betaflight() => Some(17u32),
            Self::Servo3 if fw.is_betaflight() => Some(18u32),
            Self::Soaring if fw.is_inav() => Some(47u32),
            Self::StickCommandDisable if fw.is_betaflight() => Some(41u32),
            Self::Surface if fw.is_inav() => Some(24u32),
            Self::Telemetry if fw.is_betaflight() => Some(15u32),
            Self::Telemetry if fw.is_inav() => Some(16u32),
            Self::ThreeD if fw.is_betaflight() => Some(21u32),
            Self::TurnAssist if fw.is_inav() => Some(26u32),
            Self::Turtle if fw.is_betaflight() => Some(27u32),
            Self::Turtle if fw.is_inav() => Some(43u32),
            Self::User1 if fw.is_betaflight() => Some(32u32),
            Self::User1 if fw.is_inav() => Some(37u32),
            Self::User2 if fw.is_betaflight() => Some(33u32),
            Self::User2 if fw.is_inav() => Some(38u32),
            Self::User3 if fw.is_betaflight() => Some(34u32),
            Self::User3 if fw.is_inav() => Some(48u32),
            Self::User4 if fw.is_betaflight() => Some(35u32),
            Self::VtxControlDisable if fw.is_betaflight() => Some(38u32),
            Self::VtxPitMode if fw.is_betaflight() => Some(30u32),
            _ => None,
        }
    }
}
