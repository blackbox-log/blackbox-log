#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// All currently enabled states. See [`FlagSet`][crate::units::FlagSet] and
/// [`State`].
#[allow(unused_qualifications)]
pub struct StateSet {
    firmware: crate::headers::InternalFirmware,
    raw: ::bitvec::array::BitArray<u32, ::bitvec::order::Lsb0>,
}
#[allow(unused_qualifications, clippy::cast_possible_truncation)]
impl StateSet {
    pub(crate) fn new(raw: u32, firmware: crate::headers::InternalFirmware) -> Self {
        Self {
            firmware,
            raw: ::bitvec::array::BitArray::new(raw),
        }
    }

    fn iter(&self) -> impl Iterator<Item = <Self as crate::units::FlagSet>::Flag> + '_ {
        self.raw
            .iter_ones()
            .filter_map(|bit| <State>::from_bit(bit as u32, self.firmware))
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
        self.iter()
            .map(|flag| <State as crate::units::Flag>::as_name(&flag))
            .collect()
    }
}
impl ::core::fmt::Display for StateSet {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let names = <Self as crate::units::FlagSet>::as_names(self);
        f.write_str(&names.join("|"))
    }
}
#[cfg(feature = "_serde")]
#[allow(clippy::cast_possible_truncation)]
impl ::serde::Serialize for StateSet {
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
/// A flight controller state. See [`Flag`][crate::units::Flag].
#[non_exhaustive]
pub enum State {
    /// `ACCELEROMETER_CALIBRATED`
    AccelerometerCalibrated,
    /// `AIRMODE_ACTIVE`
    AirMode,
    /// `AIRPLANE`
    Airplane,
    /// `ALTITUDE_CONTROL`
    AltitudeControl,
    /// `ANTI_WINDUP`
    AntiWindup,
    /// `ANTI_WINDUP_DEACTIVATED`
    AntiWindupDeactivated,
    /// `BOAT`
    Boat,
    /// `CALIBRATE_MAG`
    CalibrateMag,
    /// `COMPASS_CALIBRATED`
    CompassCalibrated,
    /// `ESC_SENSOR_ENABLED`
    EscSensorEnabled,
    /// `FIXED_WING_LEGACY`
    FixedWingLegacy,
    /// `FLAPERON_AVAILABLE`
    FlaperonAvailable,
    /// `FW_HEADING_USE_YAW`
    FwHeadingUseYaw,
    /// `GPS_FIX`
    GpsFix,
    /// `GPS_FIX_EVER`
    GpsFixEver,
    /// `GPS_FIX_HOME`
    GpsFixHome,
    /// `LANDING_DETECTED`
    LandingDetected,
    /// `MOVE_FORWARD_ONLY`
    MoveForwardOnly,
    /// `MULTIROTOR`
    Multirotor,
    /// `NAV_CRUISE_BRAKING`
    NavCruiseBraking,
    /// `NAV_CRUISE_BRAKING_BOOST`
    NavCruiseBrakingBoost,
    /// `NAV_CRUISE_BRAKING_LOCKED`
    NavCruiseBrakingLocked,
    /// `NAV_EXTRA_ARMING_SAFETY_BYPASSED`
    NavExtraArmingSafetyBypassed,
    /// `NAV_MOTOR_STOP_OR_IDLE`
    NavMotorStopOrIdle,
    /// `ROVER`
    Rover,
    /// `SET_REVERSIBLE_MOTORS_FORWARD`
    SetReversibleMotorsForward,
    /// `SMALL_ANGLE`
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
            Self::FixedWingLegacy => "FIXED_WING_LEGACY",
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
    unused_qualifications,
    clippy::enum_glob_use,
    clippy::match_same_arms,
    clippy::unseparated_literal_suffix
)]
impl State {
    const fn from_bit(bit: u32, fw: crate::headers::InternalFirmware) -> Option<Self> {
        use crate::headers::InternalFirmware::*;
        match (bit, fw) {
            (0u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(Self::GpsFixHome)
            }
            (1u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(Self::GpsFix)
            }
            (2u32, Betaflight4_3_0 | Betaflight4_4_0) => Some(Self::GpsFixEver),
            (2u32, Inav5_0_0) => Some(Self::CalibrateMag),
            (3u32, Inav5_0_0) => Some(Self::SmallAngle),
            (4u32, Inav5_0_0) => Some(Self::FixedWingLegacy),
            (5u32, Inav5_0_0) => Some(Self::AntiWindup),
            (6u32, Inav5_0_0) => Some(Self::FlaperonAvailable),
            (7u32, Inav5_0_0) => Some(Self::NavMotorStopOrIdle),
            (8u32, Inav5_0_0) => Some(Self::CompassCalibrated),
            (9u32, Inav5_0_0) => Some(Self::AccelerometerCalibrated),
            (11u32, Inav5_0_0) => Some(Self::NavCruiseBraking),
            (12u32, Inav5_0_0) => Some(Self::NavCruiseBrakingBoost),
            (13u32, Inav5_0_0) => Some(Self::NavCruiseBrakingLocked),
            (14u32, Inav5_0_0) => Some(Self::NavExtraArmingSafetyBypassed),
            (15u32, Inav5_0_0) => Some(Self::AirMode),
            (16u32, Inav5_0_0) => Some(Self::EscSensorEnabled),
            (17u32, Inav5_0_0) => Some(Self::Airplane),
            (18u32, Inav5_0_0) => Some(Self::Multirotor),
            (19u32, Inav5_0_0) => Some(Self::Rover),
            (20u32, Inav5_0_0) => Some(Self::Boat),
            (21u32, Inav5_0_0) => Some(Self::AltitudeControl),
            (22u32, Inav5_0_0) => Some(Self::MoveForwardOnly),
            (23u32, Inav5_0_0) => Some(Self::SetReversibleMotorsForward),
            (24u32, Inav5_0_0) => Some(Self::FwHeadingUseYaw),
            (25u32, Inav5_0_0) => Some(Self::AntiWindupDeactivated),
            (26u32, Inav5_0_0) => Some(Self::LandingDetected),
            _ => None,
        }
    }

    const fn to_bit(self, fw: crate::headers::InternalFirmware) -> Option<u32> {
        use crate::headers::InternalFirmware::*;
        match (self, fw) {
            (Self::GpsFixHome, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(0u32)
            }
            (Self::GpsFix, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(1u32)
            }
            (Self::GpsFixEver, Betaflight4_3_0 | Betaflight4_4_0) => Some(2u32),
            (Self::CalibrateMag, Inav5_0_0) => Some(2u32),
            (Self::SmallAngle, Inav5_0_0) => Some(3u32),
            (Self::FixedWingLegacy, Inav5_0_0) => Some(4u32),
            (Self::AntiWindup, Inav5_0_0) => Some(5u32),
            (Self::FlaperonAvailable, Inav5_0_0) => Some(6u32),
            (Self::NavMotorStopOrIdle, Inav5_0_0) => Some(7u32),
            (Self::CompassCalibrated, Inav5_0_0) => Some(8u32),
            (Self::AccelerometerCalibrated, Inav5_0_0) => Some(9u32),
            (Self::NavCruiseBraking, Inav5_0_0) => Some(11u32),
            (Self::NavCruiseBrakingBoost, Inav5_0_0) => Some(12u32),
            (Self::NavCruiseBrakingLocked, Inav5_0_0) => Some(13u32),
            (Self::NavExtraArmingSafetyBypassed, Inav5_0_0) => Some(14u32),
            (Self::AirMode, Inav5_0_0) => Some(15u32),
            (Self::EscSensorEnabled, Inav5_0_0) => Some(16u32),
            (Self::Airplane, Inav5_0_0) => Some(17u32),
            (Self::Multirotor, Inav5_0_0) => Some(18u32),
            (Self::Rover, Inav5_0_0) => Some(19u32),
            (Self::Boat, Inav5_0_0) => Some(20u32),
            (Self::AltitudeControl, Inav5_0_0) => Some(21u32),
            (Self::MoveForwardOnly, Inav5_0_0) => Some(22u32),
            (Self::SetReversibleMotorsForward, Inav5_0_0) => Some(23u32),
            (Self::FwHeadingUseYaw, Inav5_0_0) => Some(24u32),
            (Self::AntiWindupDeactivated, Inav5_0_0) => Some(25u32),
            (Self::LandingDetected, Inav5_0_0) => Some(26u32),
            _ => None,
        }
    }
}
