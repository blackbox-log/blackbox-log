#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// All currently enabled states. See [`FlagSet`][crate::units::FlagSet] and
/// [`State`].
#[allow(unused_qualifications)]
pub struct StateSet {
    firmware: crate::headers::Firmware,
    raw: ::bitvec::array::BitArray<[u32; 1], ::bitvec::order::Lsb0>,
}
#[allow(unused_qualifications)]
impl StateSet {
    pub(crate) fn new(raw: u32, firmware: crate::headers::Firmware) -> Self {
        Self {
            firmware,
            raw: ::bitvec::array::BitArray::new([raw]),
        }
    }
}
#[allow(unused_qualifications, clippy::cast_possible_truncation)]
impl crate::units::FlagSet for StateSet {
    type Flag = State;

    fn is_set(&self, flag: Self::Flag) -> bool {
        flag.to_bit(self.firmware)
            .map_or(false, |bit| self.raw[bit as usize])
    }

    fn as_names(&self) -> ::alloc::vec::Vec<&'static str> {
        self.raw
            .iter_ones()
            .filter_map(|bit| {
                let flag = <State>::from_bit(bit as u32, self.firmware)?;
                let name = <State as crate::units::Flag>::as_name(&flag);
                Some(name)
            })
            .collect()
    }
}
impl ::core::fmt::Display for StateSet {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let names = <Self as crate::units::FlagSet>::as_names(self);
        f.write_str(&names.join("|"))
    }
}
#[cfg(feature = "serde")]
#[allow(clippy::cast_possible_truncation)]
impl ::serde::Serialize for StateSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(None)?;
        for flag in self
            .raw
            .iter_ones()
            .filter_map(|bit| <State>::from_bit(bit as u32, self.firmware))
        {
            seq.serialize_element(&flag)?;
        }
        seq.end()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// A flight controller state. See [`Flag`][crate::units::Flag].
pub enum State {
    /// `ACCELEROMETER_CALIBRATED` (INAV only)
    AccelerometerCalibrated,
    /// `AIRMODE_ACTIVE` (INAV only)
    AirMode,
    /// `AIRPLANE` (INAV only)
    Airplane,
    /// `ALTITUDE_CONTROL` (INAV only)
    AltitudeControl,
    /// `ANTI_WINDUP` (INAV only)
    AntiWindup,
    /// `ANTI_WINDUP_DEACTIVATED` (INAV only)
    AntiWindupDeactivated,
    /// `BOAT` (INAV only)
    Boat,
    /// `CALIBRATE_MAG` (INAV only)
    CalibrateMag,
    /// `COMPASS_CALIBRATED` (INAV only)
    CompassCalibrated,
    /// `ESC_SENSOR_ENABLED` (INAV only)
    EscSensorEnabled,
    /// `FLAPERON_AVAILABLE` (INAV only)
    FlaperonAvailable,
    /// `FW_HEADING_USE_YAW` (INAV only)
    FwHeadingUseYaw,
    /// `GPS_FIX`
    GpsFix,
    /// `GPS_FIX_EVER` (Betaflight only)
    GpsFixEver,
    /// `GPS_FIX_HOME`
    GpsFixHome,
    /// `LANDING_DETECTED` (INAV only)
    LandingDetected,
    /// `MOVE_FORWARD_ONLY` (INAV only)
    MoveForwardOnly,
    /// `MULTIROTOR` (INAV only)
    Multirotor,
    /// `NAV_CRUISE_BRAKING` (INAV only)
    NavCruiseBraking,
    /// `NAV_CRUISE_BRAKING_BOOST` (INAV only)
    NavCruiseBrakingBoost,
    /// `NAV_CRUISE_BRAKING_LOCKED` (INAV only)
    NavCruiseBrakingLocked,
    /// `NAV_EXTRA_ARMING_SAFETY_BYPASSED` (INAV only)
    NavExtraArmingSafetyBypassed,
    /// `NAV_MOTOR_STOP_OR_IDLE` (INAV only)
    NavMotorStopOrIdle,
    /// `ROVER` (INAV only)
    Rover,
    /// `SET_REVERSIBLE_MOTORS_FORWARD` (INAV only)
    SetReversibleMotorsForward,
    /// `SMALL_ANGLE` (INAV only)
    SmallAngle,
}
#[allow(unused_qualifications)]
impl crate::units::Flag for State {
    fn as_name(&self) -> &'static str {
        match self {
            Self::AccelerometerCalibrated => "ACCELEROMETER_CALIBRATED",
            Self::AirMode => "AIRMODE_ACTIVE",
            Self::Airplane => "AIRPLANE",
            Self::AltitudeControl => "ALTITUDE_CONTROL",
            Self::AntiWindup => "ANTI_WINDUP",
            Self::AntiWindupDeactivated => "ANTI_WINDUP_DEACTIVATED",
            Self::Boat => "BOAT",
            Self::CalibrateMag => "CALIBRATE_MAG",
            Self::CompassCalibrated => "COMPASS_CALIBRATED",
            Self::EscSensorEnabled => "ESC_SENSOR_ENABLED",
            Self::FlaperonAvailable => "FLAPERON_AVAILABLE",
            Self::FwHeadingUseYaw => "FW_HEADING_USE_YAW",
            Self::GpsFix => "GPS_FIX",
            Self::GpsFixEver => "GPS_FIX_EVER",
            Self::GpsFixHome => "GPS_FIX_HOME",
            Self::LandingDetected => "LANDING_DETECTED",
            Self::MoveForwardOnly => "MOVE_FORWARD_ONLY",
            Self::Multirotor => "MULTIROTOR",
            Self::NavCruiseBraking => "NAV_CRUISE_BRAKING",
            Self::NavCruiseBrakingBoost => "NAV_CRUISE_BRAKING_BOOST",
            Self::NavCruiseBrakingLocked => "NAV_CRUISE_BRAKING_LOCKED",
            Self::NavExtraArmingSafetyBypassed => "NAV_EXTRA_ARMING_SAFETY_BYPASSED",
            Self::NavMotorStopOrIdle => "NAV_MOTOR_STOP_OR_IDLE",
            Self::Rover => "ROVER",
            Self::SetReversibleMotorsForward => "SET_REVERSIBLE_MOTORS_FORWARD",
            Self::SmallAngle => "SMALL_ANGLE",
        }
    }
}
impl ::core::fmt::Display for State {
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
impl State {
    const fn from_bit(bit: u32, firmware: crate::headers::Firmware) -> Option<Self> {
        use crate::headers::Firmware::{Betaflight, Inav};
        match (bit, firmware) {
            (0u32, _) => Some(Self::GpsFixHome),
            (1u32, _) => Some(Self::GpsFix),
            (2u32, Betaflight(_)) => Some(Self::GpsFixEver),
            (2u32, Inav(_)) => Some(Self::CalibrateMag),
            (3u32, Inav(_)) => Some(Self::SmallAngle),
            (5u32, Inav(_)) => Some(Self::AntiWindup),
            (6u32, Inav(_)) => Some(Self::FlaperonAvailable),
            (7u32, Inav(_)) => Some(Self::NavMotorStopOrIdle),
            (8u32, Inav(_)) => Some(Self::CompassCalibrated),
            (9u32, Inav(_)) => Some(Self::AccelerometerCalibrated),
            (11u32, Inav(_)) => Some(Self::NavCruiseBraking),
            (12u32, Inav(_)) => Some(Self::NavCruiseBrakingBoost),
            (13u32, Inav(_)) => Some(Self::NavCruiseBrakingLocked),
            (14u32, Inav(_)) => Some(Self::NavExtraArmingSafetyBypassed),
            (15u32, Inav(_)) => Some(Self::AirMode),
            (16u32, Inav(_)) => Some(Self::EscSensorEnabled),
            (17u32, Inav(_)) => Some(Self::Airplane),
            (18u32, Inav(_)) => Some(Self::Multirotor),
            (19u32, Inav(_)) => Some(Self::Rover),
            (20u32, Inav(_)) => Some(Self::Boat),
            (21u32, Inav(_)) => Some(Self::AltitudeControl),
            (22u32, Inav(_)) => Some(Self::MoveForwardOnly),
            (23u32, Inav(_)) => Some(Self::SetReversibleMotorsForward),
            (24u32, Inav(_)) => Some(Self::FwHeadingUseYaw),
            (25u32, Inav(_)) => Some(Self::AntiWindupDeactivated),
            (26u32, Inav(_)) => Some(Self::LandingDetected),
            _ => None,
        }
    }

    const fn to_bit(self, firmware: crate::headers::Firmware) -> Option<u32> {
        use crate::headers::Firmware::{Betaflight, Inav};
        match (self, firmware) {
            (Self::AccelerometerCalibrated, Inav(_)) => Some(9u32),
            (Self::AirMode, Inav(_)) => Some(15u32),
            (Self::Airplane, Inav(_)) => Some(17u32),
            (Self::AltitudeControl, Inav(_)) => Some(21u32),
            (Self::AntiWindup, Inav(_)) => Some(5u32),
            (Self::AntiWindupDeactivated, Inav(_)) => Some(25u32),
            (Self::Boat, Inav(_)) => Some(20u32),
            (Self::CalibrateMag, Inav(_)) => Some(2u32),
            (Self::CompassCalibrated, Inav(_)) => Some(8u32),
            (Self::EscSensorEnabled, Inav(_)) => Some(16u32),
            (Self::FlaperonAvailable, Inav(_)) => Some(6u32),
            (Self::FwHeadingUseYaw, Inav(_)) => Some(24u32),
            (Self::GpsFix, _) => Some(1u32),
            (Self::GpsFixEver, Betaflight(_)) => Some(2u32),
            (Self::GpsFixHome, _) => Some(0u32),
            (Self::LandingDetected, Inav(_)) => Some(26u32),
            (Self::MoveForwardOnly, Inav(_)) => Some(22u32),
            (Self::Multirotor, Inav(_)) => Some(18u32),
            (Self::NavCruiseBraking, Inav(_)) => Some(11u32),
            (Self::NavCruiseBrakingBoost, Inav(_)) => Some(12u32),
            (Self::NavCruiseBrakingLocked, Inav(_)) => Some(13u32),
            (Self::NavExtraArmingSafetyBypassed, Inav(_)) => Some(14u32),
            (Self::NavMotorStopOrIdle, Inav(_)) => Some(7u32),
            (Self::Rover, Inav(_)) => Some(19u32),
            (Self::SetReversibleMotorsForward, Inav(_)) => Some(23u32),
            (Self::SmallAngle, Inav(_)) => Some(3u32),
            _ => None,
        }
    }
}
