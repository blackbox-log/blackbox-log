use core::fmt;
use std::error;

pub trait DisarmReason: TryFrom<u32, Error = DisarmReasonError> {}

#[derive(Debug, Clone)]
pub struct DisarmReasonError;

impl error::Error for DisarmReasonError {}

impl fmt::Display for DisarmReasonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid or unsupported disarm reason")
    }
}

macro_rules! generate_disarm_reason {
    ( $( $reason:ident = $value:literal ),+ $(,)? ) => {
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
