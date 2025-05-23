#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// All currently enabled flight modes.
///
/// See [`FlagSet`][crate::units::FlagSet] and [`FlightMode`].
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
            .is_some_and(|bit| self.raw[bit as usize])
    }

    fn as_names(&self) -> ::alloc::vec::Vec<&'static str> {
        self.iter()
            .map(|flag| <FlightMode as crate::units::Flag>::as_name(&flag))
            .collect()
    }
}
#[allow(unused_qualifications)]
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
        use serde::ser::SerializeSeq as _;
        let mut seq = serializer.serialize_seq(None)?;
        for flag in self.iter() {
            seq.serialize_element(&flag)?;
        }
        seq.end()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "_serde", derive(serde::Serialize))]
/// A flight mode.
///
/// See [`Flag`][crate::units::Flag].
#[non_exhaustive]
pub enum FlightMode {
    /// `ACRO TRAINER`
    AcroTrainer,
    /// `AIRMODE`
    Airmode,
    /// `ANGLE`
    Angle,
    /// `ANGLEHOLD`
    AngleHold,
    /// `ANTI GRAVITY`
    AntiGravity,
    /// `ARM`
    Arm,
    /// `AUTOLEVEL`
    AutoLevel,
    /// `AUTOTRIM`
    AutoTrim,
    /// `AUTOTUNE`
    AutoTune,
    /// `BEEP GPS COUNT`
    BeepGpsCount,
    /// `BEEPER MUTE`
    BeeperMute,
    /// `BEEPERON`
    BeeperOn,
    /// `BEEPERMUTE`
    Beepermute,
    /// `BLACKBOX`
    Blackbox,
    /// `BLACKBOX ERASE`
    BlackboxErase,
    /// `BRAKING`
    Braking,
    /// `CALIB`
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
    /// `FLAPERON`
    Flaperon,
    /// `FPV ANGLE MIX`
    FpvAngleMix,
    /// `GCSNAV`
    GcsNav,
    /// `GIMBALCENTER`
    Gimbalcenter,
    /// `GIMBALHTRK`
    Gimbalhtrk,
    /// `GIMBALRLOCK`
    Gimbalrlock,
    /// `GIMBALTLOCK`
    Gimbaltlock,
    /// `GPS RESCUE`
    GpsRescue,
    /// `HEADADJ`
    HeadAdjust,
    /// `HEADFREE`
    HeadFree,
    /// `HEADINGHOLD`
    HeadingHold,
    /// `HOMERESET`
    HomeReset,
    /// `HORIZON`
    Horizon,
    /// `KILLSWITCH`
    Killswitch,
    /// `LAP TIMER RESET`
    LapTimerReset,
    /// `LAUNCH CONTROL`
    LaunchControl,
    /// `LEDLOW`
    LedLow,
    /// `LIGHTS`
    Lights,
    /// `LOITERDIRCHN`
    LoiterChange,
    /// `MAG`
    Mag,
    /// `MANUAL`
    Manual,
    /// `CHANGEMISSION`
    MissionChange,
    /// `MIXERPROFILE`
    Mixerprofile,
    /// `MIXERTRANSITION`
    Mixertransition,
    /// `MSPOVERRIDE`
    MspOverride,
    /// `MSPRCOVERRIDE`
    MspRcOverride,
    /// `MULTIFUNCTION`
    Multifunction,
    /// `NAV ALTHOLD`
    NavAltitudeHold,
    /// `NAVCOURSEHOLD`
    NavCourseHold,
    /// `NAV CRUISE`
    NavCruise,
    /// `NAV LAUNCH`
    NavLaunch,
    /// `NAV POSHOLD`
    NavPositionHold,
    /// `NAV RTH`
    NavRth,
    /// `NAV WP`
    NavWaypoint,
    /// `OSD`
    Osd,
    /// `OSDALT1`
    OsdAlt1,
    /// `OSDALT2`
    OsdAlt2,
    /// `OSDALT3`
    OsdAlt3,
    /// `PARALYZE`
    Paralyze,
    /// `PASSTHRU`
    Passthru,
    /// `PID AUDIO`
    PidAudio,
    /// `PLANWPMISSION`
    PlanWpMission,
    /// `PREARM`
    Prearm,
    /// `READY`
    Ready,
    /// `SERVO1`
    Servo1,
    /// `SERVO2`
    Servo2,
    /// `SERVO3`
    Servo3,
    /// `SOARING`
    Soaring,
    /// `STICK COMMAND DISABLE`
    StickCommandDisable,
    /// `SURFACE`
    Surface,
    /// `TELEMETRY`
    Telemetry,
    /// `3D`
    ThreeD,
    /// `TURNASSIST`
    TurnAssist,
    /// `TURTLE`
    Turtle,
    /// `USER1`
    User1,
    /// `USER2`
    User2,
    /// `USER3`
    User3,
    /// `USER4`
    User4,
    /// `VTX CONTROL DISABLE`
    VtxControlDisable,
    /// `VTX PIT MODE`
    VtxPitMode,
}
#[allow(unused_qualifications)]
impl crate::units::Flag for FlightMode {
    fn as_name(&self) -> &'static str {
        match self {
            Self::AcroTrainer => "ACRO TRAINER",
            Self::Airmode => "AIRMODE",
            Self::Angle => "ANGLE",
            Self::AngleHold => "ANGLEHOLD",
            Self::AntiGravity => "ANTI GRAVITY",
            Self::Arm => "ARM",
            Self::AutoLevel => "AUTOLEVEL",
            Self::AutoTrim => "AUTOTRIM",
            Self::AutoTune => "AUTOTUNE",
            Self::BeepGpsCount => "BEEP GPS COUNT",
            Self::BeeperMute => "BEEPER MUTE",
            Self::BeeperOn => "BEEPERON",
            Self::Beepermute => "BEEPERMUTE",
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
            Self::Gimbalcenter => "GIMBALCENTER",
            Self::Gimbalhtrk => "GIMBALHTRK",
            Self::Gimbalrlock => "GIMBALRLOCK",
            Self::Gimbaltlock => "GIMBALTLOCK",
            Self::GpsRescue => "GPS RESCUE",
            Self::HeadAdjust => "HEADADJ",
            Self::HeadFree => "HEADFREE",
            Self::HeadingHold => "HEADINGHOLD",
            Self::HomeReset => "HOMERESET",
            Self::Horizon => "HORIZON",
            Self::Killswitch => "KILLSWITCH",
            Self::LapTimerReset => "LAP TIMER RESET",
            Self::LaunchControl => "LAUNCH CONTROL",
            Self::LedLow => "LEDLOW",
            Self::Lights => "LIGHTS",
            Self::LoiterChange => "LOITERDIRCHN",
            Self::Mag => "MAG",
            Self::Manual => "MANUAL",
            Self::MissionChange => "CHANGEMISSION",
            Self::Mixerprofile => "MIXERPROFILE",
            Self::Mixertransition => "MIXERTRANSITION",
            Self::MspOverride => "MSPOVERRIDE",
            Self::MspRcOverride => "MSPRCOVERRIDE",
            Self::Multifunction => "MULTIFUNCTION",
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
            Self::Ready => "READY",
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
#[allow(unused_qualifications)]
impl ::core::fmt::Display for FlightMode {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let s = <Self as crate::units::Flag>::as_name(self);
        f.write_str(s)
    }
}
#[allow(
    unused_qualifications,
    clippy::enum_glob_use,
    clippy::match_same_arms,
    clippy::unseparated_literal_suffix
)]
impl FlightMode {
    const fn from_bit(bit: u32, fw: crate::headers::InternalFirmware) -> Option<Self> {
        use crate::headers::InternalFirmware::*;
        match (bit, fw) {
            (
                0u32,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5 | Inav5 | Inav6
                | Inav7 | Inav8,
            ) => Some(Self::Arm),
            (
                1u32,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5 | Inav5 | Inav6
                | Inav7 | Inav8,
            ) => Some(Self::Angle),
            (
                2u32,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5 | Inav5 | Inav6
                | Inav7 | Inav8,
            ) => Some(Self::Horizon),
            (3u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Mag)
            }
            (3u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::NavAltitudeHold),
            (4u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::HeadFree)
            }
            (4u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::HeadingHold),
            (5u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Passthru)
            }
            (5u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::HeadFree),
            (6u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Failsafe)
            }
            (6u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::HeadAdjust),
            (7u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::GpsRescue)
            }
            (7u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::CamStab),
            (8u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::AntiGravity)
            }
            (8u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::NavRth),
            (9u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::HeadAdjust)
            }
            (9u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::NavPositionHold),
            (10u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::CamStab)
            }
            (10u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Manual),
            (
                11u32,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5 | Inav5 | Inav6
                | Inav7 | Inav8,
            ) => Some(Self::BeeperOn),
            (
                12u32,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5 | Inav5 | Inav6
                | Inav7 | Inav8,
            ) => Some(Self::LedLow),
            (13u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Calibration)
            }
            (13u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Lights),
            (14u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Osd)
            }
            (14u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::NavLaunch),
            (15u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Telemetry)
            }
            (15u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Osd),
            (16u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Servo1)
            }
            (16u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Telemetry),
            (17u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Servo2)
            }
            (17u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Blackbox),
            (18u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Servo3)
            }
            (18u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Failsafe),
            (19u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Blackbox)
            }
            (19u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::NavWaypoint),
            (
                20u32,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5 | Inav5 | Inav6
                | Inav7 | Inav8,
            ) => Some(Self::Airmode),
            (21u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::ThreeD)
            }
            (21u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::HomeReset),
            (22u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::FpvAngleMix)
            }
            (22u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::GcsNav),
            (23u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::BlackboxErase)
            }
            (23u32, Inav5 | Inav6 | Inav7) => Some(Self::Killswitch),
            (24u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Camera1)
            }
            (24u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Surface),
            (25u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Camera2)
            }
            (25u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Flaperon),
            (26u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Camera3)
            }
            (26u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::TurnAssist),
            (27u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Turtle)
            }
            (27u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::AutoTrim),
            (28u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Prearm)
            }
            (28u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::AutoTune),
            (29u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::BeepGpsCount)
            }
            (29u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Camera1),
            (30u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::VtxPitMode)
            }
            (30u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Camera2),
            (31u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::Paralyze)
            }
            (31u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Camera3),
            (32u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::User1)
            }
            (32u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::OsdAlt1),
            (33u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::User2)
            }
            (33u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::OsdAlt2),
            (34u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::User3)
            }
            (34u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::OsdAlt3),
            (35u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::User4)
            }
            (35u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::NavCourseHold),
            (36u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::PidAudio)
            }
            (36u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Braking),
            (37u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::AcroTrainer)
            }
            (37u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::User1),
            (38u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::VtxControlDisable)
            }
            (38u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::User2),
            (39u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::LaunchControl)
            }
            (39u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::FpvAngleMix),
            (40u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::MspOverride)
            }
            (40u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::LoiterChange),
            (41u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::StickCommandDisable)
            }
            (41u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::MspRcOverride),
            (42u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(Self::BeeperMute)
            }
            (42u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Prearm),
            (43u32, Betaflight4_5) => Some(Self::Ready),
            (43u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Turtle),
            (44u32, Betaflight4_5) => Some(Self::LapTimerReset),
            (44u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::NavCruise),
            (45u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::AutoLevel),
            (46u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::PlanWpMission),
            (47u32, Inav5 | Inav6 | Inav7 | Inav8) => Some(Self::Soaring),
            (48u32, Inav6 | Inav7 | Inav8) => Some(Self::User3),
            (49u32, Inav6 | Inav7 | Inav8) => Some(Self::User4),
            (50u32, Inav6 | Inav7 | Inav8) => Some(Self::MissionChange),
            (51u32, Inav7 | Inav8) => Some(Self::Beepermute),
            (52u32, Inav7 | Inav8) => Some(Self::Multifunction),
            (53u32, Inav7 | Inav8) => Some(Self::Mixerprofile),
            (54u32, Inav7 | Inav8) => Some(Self::Mixertransition),
            (55u32, Inav7 | Inav8) => Some(Self::AngleHold),
            (56u32, Inav8) => Some(Self::Gimbaltlock),
            (57u32, Inav8) => Some(Self::Gimbalrlock),
            (58u32, Inav8) => Some(Self::Gimbalcenter),
            (59u32, Inav8) => Some(Self::Gimbalhtrk),
            _ => None,
        }
    }

    const fn to_bit(self, fw: crate::headers::InternalFirmware) -> Option<u32> {
        use crate::headers::InternalFirmware::*;
        match (self, fw) {
            (
                Self::Arm,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5 | Inav5 | Inav6
                | Inav7 | Inav8,
            ) => Some(0u32),
            (
                Self::Angle,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5 | Inav5 | Inav6
                | Inav7 | Inav8,
            ) => Some(1u32),
            (
                Self::Horizon,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5 | Inav5 | Inav6
                | Inav7 | Inav8,
            ) => Some(2u32),
            (Self::Mag, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(3u32)
            }
            (Self::NavAltitudeHold, Inav5 | Inav6 | Inav7 | Inav8) => Some(3u32),
            (Self::HeadFree, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(4u32)
            }
            (Self::HeadingHold, Inav5 | Inav6 | Inav7 | Inav8) => Some(4u32),
            (Self::Passthru, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(5u32)
            }
            (Self::HeadFree, Inav5 | Inav6 | Inav7 | Inav8) => Some(5u32),
            (Self::Failsafe, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(6u32)
            }
            (Self::HeadAdjust, Inav5 | Inav6 | Inav7 | Inav8) => Some(6u32),
            (Self::GpsRescue, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(7u32)
            }
            (Self::CamStab, Inav5 | Inav6 | Inav7 | Inav8) => Some(7u32),
            (Self::AntiGravity, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(8u32)
            }
            (Self::NavRth, Inav5 | Inav6 | Inav7 | Inav8) => Some(8u32),
            (Self::HeadAdjust, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(9u32)
            }
            (Self::NavPositionHold, Inav5 | Inav6 | Inav7 | Inav8) => Some(9u32),
            (Self::CamStab, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(10u32)
            }
            (Self::Manual, Inav5 | Inav6 | Inav7 | Inav8) => Some(10u32),
            (
                Self::BeeperOn,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5 | Inav5 | Inav6
                | Inav7 | Inav8,
            ) => Some(11u32),
            (
                Self::LedLow,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5 | Inav5 | Inav6
                | Inav7 | Inav8,
            ) => Some(12u32),
            (Self::Calibration, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(13u32)
            }
            (Self::Lights, Inav5 | Inav6 | Inav7 | Inav8) => Some(13u32),
            (Self::Osd, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(14u32)
            }
            (Self::NavLaunch, Inav5 | Inav6 | Inav7 | Inav8) => Some(14u32),
            (Self::Telemetry, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(15u32)
            }
            (Self::Osd, Inav5 | Inav6 | Inav7 | Inav8) => Some(15u32),
            (Self::Servo1, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(16u32)
            }
            (Self::Telemetry, Inav5 | Inav6 | Inav7 | Inav8) => Some(16u32),
            (Self::Servo2, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(17u32)
            }
            (Self::Blackbox, Inav5 | Inav6 | Inav7 | Inav8) => Some(17u32),
            (Self::Servo3, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(18u32)
            }
            (Self::Failsafe, Inav5 | Inav6 | Inav7 | Inav8) => Some(18u32),
            (Self::Blackbox, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(19u32)
            }
            (Self::NavWaypoint, Inav5 | Inav6 | Inav7 | Inav8) => Some(19u32),
            (
                Self::Airmode,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5 | Inav5 | Inav6
                | Inav7 | Inav8,
            ) => Some(20u32),
            (Self::ThreeD, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(21u32)
            }
            (Self::HomeReset, Inav5 | Inav6 | Inav7 | Inav8) => Some(21u32),
            (Self::FpvAngleMix, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(22u32)
            }
            (Self::GcsNav, Inav5 | Inav6 | Inav7 | Inav8) => Some(22u32),
            (
                Self::BlackboxErase,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5,
            ) => Some(23u32),
            (Self::Killswitch, Inav5 | Inav6 | Inav7) => Some(23u32),
            (Self::Camera1, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(24u32)
            }
            (Self::Surface, Inav5 | Inav6 | Inav7 | Inav8) => Some(24u32),
            (Self::Camera2, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(25u32)
            }
            (Self::Flaperon, Inav5 | Inav6 | Inav7 | Inav8) => Some(25u32),
            (Self::Camera3, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(26u32)
            }
            (Self::TurnAssist, Inav5 | Inav6 | Inav7 | Inav8) => Some(26u32),
            (Self::Turtle, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(27u32)
            }
            (Self::AutoTrim, Inav5 | Inav6 | Inav7 | Inav8) => Some(27u32),
            (Self::Prearm, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(28u32)
            }
            (Self::AutoTune, Inav5 | Inav6 | Inav7 | Inav8) => Some(28u32),
            (Self::BeepGpsCount, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(29u32)
            }
            (Self::Camera1, Inav5 | Inav6 | Inav7 | Inav8) => Some(29u32),
            (Self::VtxPitMode, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(30u32)
            }
            (Self::Camera2, Inav5 | Inav6 | Inav7 | Inav8) => Some(30u32),
            (Self::Paralyze, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(31u32)
            }
            (Self::Camera3, Inav5 | Inav6 | Inav7 | Inav8) => Some(31u32),
            (Self::User1, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(32u32)
            }
            (Self::OsdAlt1, Inav5 | Inav6 | Inav7 | Inav8) => Some(32u32),
            (Self::User2, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(33u32)
            }
            (Self::OsdAlt2, Inav5 | Inav6 | Inav7 | Inav8) => Some(33u32),
            (Self::User3, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(34u32)
            }
            (Self::OsdAlt3, Inav5 | Inav6 | Inav7 | Inav8) => Some(34u32),
            (Self::User4, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(35u32)
            }
            (Self::NavCourseHold, Inav5 | Inav6 | Inav7 | Inav8) => Some(35u32),
            (Self::PidAudio, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(36u32)
            }
            (Self::Braking, Inav5 | Inav6 | Inav7 | Inav8) => Some(36u32),
            (Self::AcroTrainer, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(37u32)
            }
            (Self::User1, Inav5 | Inav6 | Inav7 | Inav8) => Some(37u32),
            (
                Self::VtxControlDisable,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5,
            ) => Some(38u32),
            (Self::User2, Inav5 | Inav6 | Inav7 | Inav8) => Some(38u32),
            (
                Self::LaunchControl,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5,
            ) => Some(39u32),
            (Self::FpvAngleMix, Inav5 | Inav6 | Inav7 | Inav8) => Some(39u32),
            (Self::MspOverride, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(40u32)
            }
            (Self::LoiterChange, Inav5 | Inav6 | Inav7 | Inav8) => Some(40u32),
            (
                Self::StickCommandDisable,
                Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5,
            ) => Some(41u32),
            (Self::MspRcOverride, Inav5 | Inav6 | Inav7 | Inav8) => Some(41u32),
            (Self::BeeperMute, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => {
                Some(42u32)
            }
            (Self::Prearm, Inav5 | Inav6 | Inav7 | Inav8) => Some(42u32),
            (Self::Ready, Betaflight4_5) => Some(43u32),
            (Self::Turtle, Inav5 | Inav6 | Inav7 | Inav8) => Some(43u32),
            (Self::LapTimerReset, Betaflight4_5) => Some(44u32),
            (Self::NavCruise, Inav5 | Inav6 | Inav7 | Inav8) => Some(44u32),
            (Self::AutoLevel, Inav5 | Inav6 | Inav7 | Inav8) => Some(45u32),
            (Self::PlanWpMission, Inav5 | Inav6 | Inav7 | Inav8) => Some(46u32),
            (Self::Soaring, Inav5 | Inav6 | Inav7 | Inav8) => Some(47u32),
            (Self::User3, Inav6 | Inav7 | Inav8) => Some(48u32),
            (Self::User4, Inav6 | Inav7 | Inav8) => Some(49u32),
            (Self::MissionChange, Inav6 | Inav7 | Inav8) => Some(50u32),
            (Self::Beepermute, Inav7 | Inav8) => Some(51u32),
            (Self::Multifunction, Inav7 | Inav8) => Some(52u32),
            (Self::Mixerprofile, Inav7 | Inav8) => Some(53u32),
            (Self::Mixertransition, Inav7 | Inav8) => Some(54u32),
            (Self::AngleHold, Inav7 | Inav8) => Some(55u32),
            (Self::Gimbaltlock, Inav8) => Some(56u32),
            (Self::Gimbalrlock, Inav8) => Some(57u32),
            (Self::Gimbalcenter, Inav8) => Some(58u32),
            (Self::Gimbalhtrk, Inav8) => Some(59u32),
            _ => None,
        }
    }
}
