#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "_serde", derive(serde::Serialize))]
pub enum PwmProtocol {
    /// `BRUSHED`
    Brushed,
    /// `DISABLED` (Betaflight only)
    Disabled,
    /// `DSHOT150`
    Dshot150,
    /// `DSHOT300`
    Dshot300,
    /// `DSHOT600`
    Dshot600,
    /// `MULTISHOT`
    Multishot,
    /// `ONESHOT125`
    Oneshot125,
    /// `ONESHOT42` (Betaflight only)
    Oneshot42,
    /// `PROSHOT1000` (Betaflight only)
    Proshot1000,
    /// `STANDARD`
    Standard,
    Unknown,
}
#[allow(unused_qualifications)]
impl crate::units::Flag for PwmProtocol {
    fn as_name(&self) -> &'static str {
        match self {
            Self::Brushed => "BRUSHED",
            Self::Disabled => "DISABLED",
            Self::Dshot150 => "DSHOT150",
            Self::Dshot300 => "DSHOT300",
            Self::Dshot600 => "DSHOT600",
            Self::Multishot => "MULTISHOT",
            Self::Oneshot125 => "ONESHOT125",
            Self::Oneshot42 => "ONESHOT42",
            Self::Proshot1000 => "PROSHOT1000",
            Self::Standard => "STANDARD",
            Self::Unknown => "UNKNOWN",
        }
    }
}
impl ::core::fmt::Display for PwmProtocol {
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
impl PwmProtocol {
    pub(crate) fn new(raw: u32, firmware: crate::headers::Firmware) -> Self {
        use crate::headers::Firmware::{Betaflight, Inav};
        match (raw, firmware) {
            (0u32, _) => Self::Standard,
            (1u32, _) => Self::Oneshot125,
            (2u32, Betaflight(_)) => Self::Oneshot42,
            (2u32, Inav(_)) => Self::Multishot,
            (3u32, Betaflight(_)) => Self::Multishot,
            (3u32, Inav(_)) => Self::Brushed,
            (4u32, Betaflight(_)) => Self::Brushed,
            (4u32, Inav(_)) => Self::Dshot150,
            (5u32, Betaflight(_)) => Self::Dshot150,
            (5u32, Inav(_)) => Self::Dshot300,
            (6u32, Betaflight(_)) => Self::Dshot300,
            (6u32, Inav(_)) => Self::Dshot600,
            (7u32, Betaflight(_)) => Self::Dshot600,
            (8u32, Betaflight(_)) => Self::Proshot1000,
            (9u32, Betaflight(_)) => Self::Disabled,
            _ => {
                #[allow(clippy::redundant_closure_call)]
                (|raw| tracing::debug!("invalid pwm protocol ({raw})"))(raw);
                Self::Unknown
            }
        }
    }
}
