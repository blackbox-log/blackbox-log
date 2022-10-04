use alloc::borrow::ToOwned;
use core::fmt;
use core::str::FromStr;

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
    Baseflight,
    Cleanflight,
    INav,
}

impl FromStr for FirmwareKind {
    type Err = crate::parser::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "cleanflight" => Ok(Self::Cleanflight),
            "baseflight" => Ok(Self::Baseflight),
            "inav" => Ok(Self::INav),
            _ => Err(crate::parser::ParseError::UnknownFirmware(s.to_owned())),
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
