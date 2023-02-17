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
    unused_qualifications,
    clippy::match_same_arms,
    clippy::unseparated_literal_suffix,
    clippy::wildcard_enum_match_arm
)]
impl State {
    const fn from_bit(bit: u32, fw: crate::headers::InternalFirmware) -> Option<Self> {
        match bit {
            0u32 => Some(Self::GpsFixHome),
            1u32 => Some(Self::GpsFix),
            2u32 if fw.is_betaflight() => Some(Self::GpsFixEver),
            2u32 if fw.is_inav() => Some(Self::CalibrateMag),
            3u32 if fw.is_inav() => Some(Self::SmallAngle),
            5u32 if fw.is_inav() => Some(Self::AntiWindup),
            6u32 if fw.is_inav() => Some(Self::FlaperonAvailable),
            7u32 if fw.is_inav() => Some(Self::NavMotorStopOrIdle),
            8u32 if fw.is_inav() => Some(Self::CompassCalibrated),
            9u32 if fw.is_inav() => Some(Self::AccelerometerCalibrated),
            11u32 if fw.is_inav() => Some(Self::NavCruiseBraking),
            12u32 if fw.is_inav() => Some(Self::NavCruiseBrakingBoost),
            13u32 if fw.is_inav() => Some(Self::NavCruiseBrakingLocked),
            14u32 if fw.is_inav() => Some(Self::NavExtraArmingSafetyBypassed),
            15u32 if fw.is_inav() => Some(Self::AirMode),
            16u32 if fw.is_inav() => Some(Self::EscSensorEnabled),
            17u32 if fw.is_inav() => Some(Self::Airplane),
            18u32 if fw.is_inav() => Some(Self::Multirotor),
            19u32 if fw.is_inav() => Some(Self::Rover),
            20u32 if fw.is_inav() => Some(Self::Boat),
            21u32 if fw.is_inav() => Some(Self::AltitudeControl),
            22u32 if fw.is_inav() => Some(Self::MoveForwardOnly),
            23u32 if fw.is_inav() => Some(Self::SetReversibleMotorsForward),
            24u32 if fw.is_inav() => Some(Self::FwHeadingUseYaw),
            25u32 if fw.is_inav() => Some(Self::AntiWindupDeactivated),
            26u32 if fw.is_inav() => Some(Self::LandingDetected),
            _ => None,
        }
    }

    const fn to_bit(self, fw: crate::headers::InternalFirmware) -> Option<u32> {
        match self {
            Self::AccelerometerCalibrated if fw.is_inav() => Some(9u32),
            Self::AirMode if fw.is_inav() => Some(15u32),
            Self::Airplane if fw.is_inav() => Some(17u32),
            Self::AltitudeControl if fw.is_inav() => Some(21u32),
            Self::AntiWindup if fw.is_inav() => Some(5u32),
            Self::AntiWindupDeactivated if fw.is_inav() => Some(25u32),
            Self::Boat if fw.is_inav() => Some(20u32),
            Self::CalibrateMag if fw.is_inav() => Some(2u32),
            Self::CompassCalibrated if fw.is_inav() => Some(8u32),
            Self::EscSensorEnabled if fw.is_inav() => Some(16u32),
            Self::FlaperonAvailable if fw.is_inav() => Some(6u32),
            Self::FwHeadingUseYaw if fw.is_inav() => Some(24u32),
            Self::GpsFix => Some(1u32),
            Self::GpsFixEver if fw.is_betaflight() => Some(2u32),
            Self::GpsFixHome => Some(0u32),
            Self::LandingDetected if fw.is_inav() => Some(26u32),
            Self::MoveForwardOnly if fw.is_inav() => Some(22u32),
            Self::Multirotor if fw.is_inav() => Some(18u32),
            Self::NavCruiseBraking if fw.is_inav() => Some(11u32),
            Self::NavCruiseBrakingBoost if fw.is_inav() => Some(12u32),
            Self::NavCruiseBrakingLocked if fw.is_inav() => Some(13u32),
            Self::NavExtraArmingSafetyBypassed if fw.is_inav() => Some(14u32),
            Self::NavMotorStopOrIdle if fw.is_inav() => Some(7u32),
            Self::Rover if fw.is_inav() => Some(19u32),
            Self::SetReversibleMotorsForward if fw.is_inav() => Some(23u32),
            Self::SmallAngle if fw.is_inav() => Some(3u32),
            _ => None,
        }
    }
}
