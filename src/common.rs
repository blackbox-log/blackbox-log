use alloc::borrow::ToOwned;
use core::fmt;
use core::marker::PhantomData;
use core::str::FromStr;

use crate::parser::ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogVersion {
    V1,
    V2,
}

impl FromStr for LogVersion {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "1" | "v1" => Ok(Self::V1),
            "2" | "v2" => Ok(Self::V2),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FirmwareKind {
    Betaflight,
    INav,
}

impl FromStr for FirmwareKind {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "cleanflight" | "betaflight" => Ok(Self::Betaflight),
            "baseflight" => {
                tracing::error!("Baseflight logs are not supported");
                Err(ParseError::UnknownFirmware(s.to_owned()))
            }
            "inav" => Ok(Self::INav),
            _ => Err(ParseError::UnknownFirmware(s.to_owned())),
        }
    }
}

pub trait DisarmReason: TryFrom<u32, Error = DisarmReasonError> {}

#[derive(Debug, Clone)]
pub struct DisarmReasonError;

#[cfg(feature = "std")]
impl std::error::Error for DisarmReasonError {}

impl fmt::Display for DisarmReasonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid or unsupported disarm reason")
    }
}

macro_rules! generate_disarm_reason {
    ( $( $reason:ident = $value:literal ),+ $(,)? ) => {
        #[non_exhaustive]
        pub enum DisarmReason {
            $( $reason = $value ),+
        }

        impl crate::common::DisarmReason for DisarmReason {}

        impl TryFrom<u32> for DisarmReason {
            type Error = crate::common::DisarmReasonError;

            fn try_from(reason: u32) -> Result<Self, Self::Error> {
                match reason {
                    $( $value => Ok(Self::$reason), )+
                    _ => Err(crate::common::DisarmReasonError),
                }
            }
        }
    }
}

macro_rules! generate_flight_mode {
    ( $( $mode:ident / $mode_fn:ident = $bit:expr ),+ $(,)? ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u32)]
        pub enum FlightMode {
            $( $mode = 1 << $bit ),+
        }

        impl FlightMode {
            const fn to_bit(self) -> u32 {
                match self {
                    $( Self::$mode => $bit ),+
                }
            }
        }

        impl crate::common::FlightModeFlags<FlightMode> {
            #[inline]
            pub const fn is_mode_set(self, mode: FlightMode) -> bool {
                self.is_bit_set(mode.to_bit())
            }

            $(
                pub const fn $mode_fn(self) -> bool {
                    self.is_mode_set(FlightMode::$mode)
                }
            )+

            pub fn to_modes(self) -> alloc::vec::Vec<FlightMode> {
                [ $( FlightMode::$mode ),+ ]
                    .into_iter()
                    .filter(|&mode| self.is_mode_set(mode))
                    .collect()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlightModeFlags<T>(u32, PhantomData<T>);

impl<T> FlightModeFlags<T> {
    pub const fn new(flags: u32) -> Self {
        Self(flags, PhantomData)
    }

    #[inline]
    pub(crate) const fn is_bit_set(&self, bit: u32) -> bool {
        (self.0 & (1 << bit)) > 0
    }
}
