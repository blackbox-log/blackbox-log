use alloc::vec::Vec;
use core::fmt;

use bitvec::prelude::*;
pub use uom::si;

use crate::common::FirmwareKind;

pub(crate) mod prelude {
    pub use super::si::acceleration::meter_per_second_squared as mps2;
    pub use super::si::angular_velocity::degree_per_second;
    pub use super::si::electric_current::{ampere, milliampere};
    pub use super::si::electric_potential::{millivolt, volt};
    pub use super::si::time::{microsecond, second};
    pub use super::{Acceleration, AngularVelocity, ElectricCurrent, ElectricPotential, Time};
}

pub mod system {
    use uom::system;
    uom::ISQ!(
        uom::si,
        f64,
        (
            meter,
            gram,
            microsecond,
            milliampere,
            degree_celsius,
            mole,
            candela
        )
    );
}

pub use self::system::{Acceleration, AngularVelocity, ElectricCurrent, ElectricPotential, Time};

pub trait FlagSet {
    type Flag: Flag;

    fn is_set(&self, flag: Self::Flag) -> bool;
    fn as_names(&self) -> Vec<&'static str>;
}

pub trait Flag {
    fn as_name(&self) -> &'static str;
}

macro_rules! define_flag_set {
    ($set:ident, $flag_name:ident {
        $( $flag:ident : $($beta:literal)? / $($inav:literal)? ),* $(,)?
    }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $set {
            firmware: FirmwareKind,
            raw: BitArray<[u32; 1], Lsb0>,
        }

        impl $set {
            pub(crate) fn new(raw: u32, firmware: FirmwareKind) -> Self {
                Self {
                    firmware,
                    raw: BitArray::new([raw]),
                }
            }
        }

        impl FlagSet for $set {
            type Flag = $flag_name;

            fn is_set(&self, flag: Self::Flag) -> bool {
                flag.to_bit(self.firmware)
                    .map_or(false, |bit| self.raw[bit])
            }

            fn as_names(&self) -> Vec<&'static str> {
                self.raw
                    .iter_ones()
                    .filter_map(|bit| Some($flag_name::from_bit(bit, self.firmware)?.as_name()))
                    .collect()
            }
        }

        impl fmt::Display for $set {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.as_names().join("|"))
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $flag_name {
            $( $flag ),*
        }

        impl Flag for $flag_name {
            fn as_name(&self) -> &'static str {
                match self {
                    $( Self::$flag => stringify!($flag) ),*
                }
            }
        }

        impl $flag_name {
            const fn from_bit(bit: usize, firmware: FirmwareKind) -> Option<Self> {
                match (bit, firmware) {
                    $($( ($beta, FirmwareKind::Betaflight) => Some(Self::$flag), )?)*
                    $($( ($inav, FirmwareKind::INav) => Some(Self::$flag), )?)*
                    _ => None,
                }
            }

            const fn to_bit(self, firmware: FirmwareKind) -> Option<usize> {
                match (self, firmware) {
                    $($( (Self::$flag, FirmwareKind::Betaflight) => Some($beta), )?)*
                    $($( (Self::$flag, FirmwareKind::INav) => Some($inav), )?)*
                    _ => None,
                }
            }
        }

        impl fmt::Display for $flag_name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_name())
            }
        }
    };
}

define_flag_set!(FlightModeSet, FlightMode {
    Angle:     1 /  0,
    Horizon:   2 /  1,
    HeadFree:  4 /  6,
    Failsafe:  6 /  9,
    Turtle:   27 / 15,

    Arm:            0 / ,
    Mag:            3 / ,
    Passthru:       5 / ,
    GpsRescue:      7 / ,
    Antigravity:    8 / ,
    HeadAdjust:     9 / ,
    CamStab:       10 / ,
    BeeperOn:      11 / ,
    LedLow:        12 / ,
    Calib:         13 / ,
    Osd:           14 / ,
    Telemetry:     15 / ,
    Servo1:        16 / ,
    Servo2:        17 / ,
    Servo3:        18 / ,
    Blackbox:      19 / ,
    Airmode:       20 / ,
    ThreeD:        21 / ,
    FpvAngleMix:   22 / ,
    BlackboxErase: 23 / ,
    Camera1:       24 / ,
    Camera2:       25 / ,
    Camera3:       26 / ,
    Prearm:        28 / ,
    BeepGpsCount:  29 / ,
    VtxPitmode:    30 / ,
    Paralyze:      31 / ,

    // User1:               32 / ,
    // User2:               33 / ,
    // User3:               34 / ,
    // User4:               35 / ,
    // PidAudio:            36 / ,
    // AcroTrainer:         37 / ,
    // VtxControlDisable:   38 / ,
    // LaunchControl:       39 / ,
    // MspOverride:         40 / ,
    // StickCommandDisable: 41 / ,
    // BeeperMute:          42 / ,

    Heading:       /  2,
    NavAltHold:    /  3,
    NavRth:        /  4,
    NavPoshold:    /  5,
    NavLaunch:     /  7,
    Manual:        /  8,
    AutoTune:      / 10,
    NavWp:         / 11,
    NavCourseHold: / 12,
    Flaperon:      / 13,
    TurnAssistant: / 14,
    Soaring:       / 16,
});

define_flag_set!(StateSet, State {
    GpsFixHome: 0 / 0,
    GpsFix:     1 / 1,
    GpsFixEver: 2 /  ,

    CalibrateMag:                 /  2,
    SmallAngle:                   /  3,
    FixedWingLegacy:              /  4,
    AntiWindup:                   /  5,
    FlaperonAvailable:            /  6,
    NavMotorStopOrIdle:           /  7,
    CompassCalibrated:            /  8,
    AccelerometerCalibrated:      /  9,
    NavCruiseBraking:             / 11,
    NavCruiseBrakingBoost:        / 12,
    NavCruiseBrakingLocked:       / 13,
    NavExtraArmingSafetyBypassed: / 14,
    AirmodeActive:                / 15,
    EscSensorEnabled:             / 16,
    Airplane:                     / 17,
    Multirotor:                   / 18,
    Rover:                        / 19,
    Boat:                         / 20,
    AltitudeControl:              / 21,
    MoveForwardOnly:              / 22,
    SetReversibleMotorsForward:   / 23,
    FwHeadingUseYaw:              / 24,
    AntiWindupDeactivated:        / 25,
    LandingDetected:              / 26,
});

define_flag_set!(FailsafePhaseSet, FailsafePhase {
    Idle:             0 / 0,
    RxLossDetected:   1 / 1,
    RxLossIdle:         / 2,
    ReturnToHome:       / 3,
    Landing:          2 / 4,
    Landed:           3 / 5,
    RxLossMonitoring: 4 / 6,
    RxLossRecovered:  5 / 7,
    GpsRescue:        6 /  ,
});

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! float_eq {
        ($left:expr, $right:expr) => {
            assert!(
                ($left - $right).abs() < 0.01,
                "floats are greater than 0.01 apart"
            );
        };
    }

    #[test]
    fn time_1_microsecond() {
        use si::time::microsecond;
        assert_eq!(1., Time::new::<microsecond>(1.).get::<microsecond>());
    }

    #[test]
    fn time_1_day() {
        use si::time::day;
        assert_eq!(1., Time::new::<day>(1.).get::<day>());
    }

    #[test]
    fn acceleration_1_mm_per_sec_sq() {
        use si::acceleration::millimeter_per_second_squared as mmps2;
        float_eq!(1., Acceleration::new::<mmps2>(1.).get::<mmps2>());
    }

    #[test]
    fn acceleration_1_km_per_sec_sq() {
        use si::acceleration::kilometer_per_second_squared as kmps2;
        float_eq!(1., Acceleration::new::<kmps2>(1.).get::<kmps2>());
    }

    #[test]
    fn angular_velocity_1_rev_per_sec() {
        use si::angular_velocity::revolution_per_second as rps;
        float_eq!(1., AngularVelocity::new::<rps>(1.).get::<rps>());
    }

    #[test]
    fn angular_velocity_5k_degree() {
        use si::angular_velocity::degree_per_second;
        float_eq!(
            5_000.,
            AngularVelocity::new::<degree_per_second>(5_000.).get::<degree_per_second>()
        );
    }

    #[test]
    fn electric_current_1_milliamp() {
        use si::electric_current::milliampere;
        float_eq!(
            1.,
            ElectricCurrent::new::<milliampere>(1.).get::<milliampere>()
        );
    }

    #[test]
    fn electric_current_1_kiloamp() {
        use si::electric_current::kiloampere;
        float_eq!(
            1.,
            ElectricCurrent::new::<kiloampere>(1.).get::<kiloampere>()
        );
    }

    #[test]
    fn electric_potential_1_millivolt() {
        use si::electric_potential::millivolt;
        float_eq!(
            1.,
            ElectricPotential::new::<millivolt>(1.).get::<millivolt>()
        );
    }

    #[test]
    fn electric_potential_1_kilovolt() {
        use si::electric_potential::kilovolt;
        float_eq!(1., ElectricPotential::new::<kilovolt>(1.).get::<kilovolt>());
    }
}
