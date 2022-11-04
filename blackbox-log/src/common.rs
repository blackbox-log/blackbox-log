use alloc::borrow::ToOwned;
use core::fmt;
use core::str::FromStr;

use crate::parser::ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LogVersion {
    V2,
}

impl FromStr for LogVersion {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "2" | "v2" => Ok(Self::V2),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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

// TODO: waiting on https://github.com/rust-lang/rust-clippy/pull/9545 to land
#[allow(clippy::std_instead_of_core)]
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

macro_rules! byte_enum {
    (
        $( #[$attr:meta] )*
        $pub:vis enum $name:ident {
            $(
                $( #[$variant_attr:meta] )*
                $variant:ident = $value:expr
            ),+
            $(,)?
        }
    ) => {
        $( #[$attr] )*
        $pub enum $name {
            $( $( #[$variant_attr] )* $variant = $value ),+
        }

        impl $name {
            #[allow(dead_code)]
            pub(crate) const fn from_byte(byte: u8) -> Option<Self> {
                match byte {
                    $( $value => Some(Self::$variant), )+
                    _ => None,
                }
            }

            #[allow(dead_code)]
            pub(crate) fn from_num_str(s: &str) -> Option<Self> {
                match s {
                    $( stringify!($value) => Some(Self::$variant), )+
                    _ => None,
                }
            }
        }

        impl From<$name> for u8 {
            fn from(from: $name) -> u8 {
                match from {
                    $( $name::$variant => $value ),+
                }
            }
        }
    }
}
