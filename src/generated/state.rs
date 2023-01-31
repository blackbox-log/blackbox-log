#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// All currently enabled states. See [`FlagSet`][`crate::units::FlagSet`] and
/// [`State`].
#[allow(unused_qualifications)]
pub struct StateSet {
    firmware: crate::headers::FirmwareKind,
    raw: ::bitvec::array::BitArray<[u32; 1], ::bitvec::order::Lsb0>,
}
#[allow(unused_qualifications)]
impl StateSet {
    pub(crate) fn new(raw: u32, firmware: crate::headers::FirmwareKind) -> Self {
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
            .filter_map(|bit| Some(<State>::from_bit(bit as u32, self.firmware)?.as_name()))
            .collect()
    }
}
impl ::core::fmt::Display for StateSet {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.write_str(&self.as_names().join("|"))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A flight controller state. See [`Flag`][`crate::units::Flag`].
pub enum State {
    /// `ACCELEROMETER_CALIBRATED` (INAV only)
    AccelerometerCalibrated,
    /// `AIRMODE_ACTIVE` (INAV only)
    Airmode,
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
            Self::Airmode => "AIRMODE_ACTIVE",
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
        f.write_str(self.as_name())
    }
}
#[allow(
    unused_qualifications,
    clippy::match_same_arms,
    clippy::unseparated_literal_suffix
)]
impl State {
    const fn from_bit(bit: u32, firmware: crate::headers::FirmwareKind) -> Option<Self> {
        use crate::headers::FirmwareKind::{Betaflight, Inav};
        match (bit, firmware) {
            (0u32, _) => Some(Self::GpsFixHome),
            (1u32, _) => Some(Self::GpsFix),
            (2u32, Betaflight) => Some(Self::GpsFixEver),
            (2u32, Inav) => Some(Self::CalibrateMag),
            (3u32, Inav) => Some(Self::SmallAngle),
            (5u32, Inav) => Some(Self::AntiWindup),
            (6u32, Inav) => Some(Self::FlaperonAvailable),
            (7u32, Inav) => Some(Self::NavMotorStopOrIdle),
            (8u32, Inav) => Some(Self::CompassCalibrated),
            (9u32, Inav) => Some(Self::AccelerometerCalibrated),
            (11u32, Inav) => Some(Self::NavCruiseBraking),
            (12u32, Inav) => Some(Self::NavCruiseBrakingBoost),
            (13u32, Inav) => Some(Self::NavCruiseBrakingLocked),
            (14u32, Inav) => Some(Self::NavExtraArmingSafetyBypassed),
            (15u32, Inav) => Some(Self::Airmode),
            (16u32, Inav) => Some(Self::EscSensorEnabled),
            (17u32, Inav) => Some(Self::Airplane),
            (18u32, Inav) => Some(Self::Multirotor),
            (19u32, Inav) => Some(Self::Rover),
            (20u32, Inav) => Some(Self::Boat),
            (21u32, Inav) => Some(Self::AltitudeControl),
            (22u32, Inav) => Some(Self::MoveForwardOnly),
            (23u32, Inav) => Some(Self::SetReversibleMotorsForward),
            (24u32, Inav) => Some(Self::FwHeadingUseYaw),
            (25u32, Inav) => Some(Self::AntiWindupDeactivated),
            (26u32, Inav) => Some(Self::LandingDetected),
            _ => None,
        }
    }

    const fn to_bit(self, firmware: crate::headers::FirmwareKind) -> Option<u32> {
        use crate::headers::FirmwareKind::{Betaflight, Inav};
        match (self, firmware) {
            (Self::AccelerometerCalibrated, Inav) => Some(9u32),
            (Self::Airmode, Inav) => Some(15u32),
            (Self::Airplane, Inav) => Some(17u32),
            (Self::AltitudeControl, Inav) => Some(21u32),
            (Self::AntiWindup, Inav) => Some(5u32),
            (Self::AntiWindupDeactivated, Inav) => Some(25u32),
            (Self::Boat, Inav) => Some(20u32),
            (Self::CalibrateMag, Inav) => Some(2u32),
            (Self::CompassCalibrated, Inav) => Some(8u32),
            (Self::EscSensorEnabled, Inav) => Some(16u32),
            (Self::FlaperonAvailable, Inav) => Some(6u32),
            (Self::FwHeadingUseYaw, Inav) => Some(24u32),
            (Self::GpsFix, _) => Some(1u32),
            (Self::GpsFixEver, Betaflight) => Some(2u32),
            (Self::GpsFixHome, _) => Some(0u32),
            (Self::LandingDetected, Inav) => Some(26u32),
            (Self::MoveForwardOnly, Inav) => Some(22u32),
            (Self::Multirotor, Inav) => Some(18u32),
            (Self::NavCruiseBraking, Inav) => Some(11u32),
            (Self::NavCruiseBrakingBoost, Inav) => Some(12u32),
            (Self::NavCruiseBrakingLocked, Inav) => Some(13u32),
            (Self::NavExtraArmingSafetyBypassed, Inav) => Some(14u32),
            (Self::NavMotorStopOrIdle, Inav) => Some(7u32),
            (Self::Rover, Inav) => Some(19u32),
            (Self::SetReversibleMotorsForward, Inav) => Some(23u32),
            (Self::SmallAngle, Inav) => Some(3u32),
            _ => None,
        }
    }
}
