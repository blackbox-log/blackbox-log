use alloc::vec::Vec;
use core::fmt;

use bitvec::prelude::*;

use crate::common::FirmwareKind;
use crate::parser::headers::CurrentMeterConfig;
use crate::parser::{as_signed, Headers};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Amperage {
    raw: i32,
    current_meter: CurrentMeterConfig,
}

impl Amperage {
    pub(crate) fn new(raw: u32, headers: &Headers) -> Self {
        Self {
            raw: as_signed(raw),
            current_meter: headers.current_meter.unwrap(),
        }
    }

    pub fn as_amps(&self) -> f64 {
        let milliamps = f64::from(self.raw * 3300) / 4095.;
        let milliamps = milliamps - f64::from(self.current_meter.offset);
        (milliamps * 10.) / f64::from(self.current_meter.scale)
    }
}

impl fmt::Display for Amperage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.*}", f.precision().unwrap_or(2), self.as_amps())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Voltage {
    pub(crate) raw: u32,
    pub(crate) scale: u16,
}

impl Voltage {
    pub(crate) fn new(raw: u32, headers: &Headers) -> Self {
        Self {
            raw,
            scale: headers.vbat.unwrap().scale,
        }
    }

    pub fn as_volts(&self) -> f64 {
        f64::from(
            self.raw
                .saturating_mul(330)
                .saturating_mul(u32::from(self.scale)),
        ) / 4.095
    }
}

impl fmt::Display for Voltage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.*}", f.precision().unwrap_or(2), self.as_volts())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Acceleration {
    raw: i32,
    one_g: u16,
}

impl Acceleration {
    pub(crate) fn new(raw: u32, headers: &Headers) -> Self {
        Self {
            raw: as_signed(raw),
            one_g: headers.acceleration_1g.unwrap(),
        }
    }

    pub fn as_gs(&self) -> f64 {
        f64::from(self.raw) / f64::from(self.one_g)
    }
}

impl fmt::Display for Acceleration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.*}", f.precision().unwrap_or(2), self.as_gs())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rotation(i32);

impl Rotation {
    pub(crate) const fn new(raw: u32) -> Self {
        Self(as_signed(raw))
    }

    pub fn as_degrees(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for Rotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
