#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "_serde", derive(serde::Serialize))]
/// An ESC communication protocol.
#[non_exhaustive]
pub enum PwmProtocol {
    /// `BRUSHED`
    Brushed,
    /// `DISABLED`
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
    /// `ONESHOT42`
    Oneshot42,
    /// `PROSHOT1000`
    Proshot1000,
    /// `STANDARD`
    Standard,
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
        }
    }
}
#[allow(unused_qualifications)]
impl ::core::fmt::Display for PwmProtocol {
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
impl PwmProtocol {
    pub(crate) fn new(raw: u32, fw: crate::headers::InternalFirmware) -> Option<Self> {
        use crate::headers::InternalFirmware::*;
        match (raw, fw) {
            (0u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Inav5 | Inav6 | Inav7) => {
                Some(Self::Standard)
            }
            (1u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4 | Inav5 | Inav6 | Inav7) => {
                Some(Self::Oneshot125)
            }
            (2u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4) => Some(Self::Oneshot42),
            (2u32, Inav5 | Inav6 | Inav7) => Some(Self::Multishot),
            (3u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4) => Some(Self::Multishot),
            (3u32, Inav5 | Inav6 | Inav7) => Some(Self::Brushed),
            (4u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4) => Some(Self::Brushed),
            (4u32, Inav5 | Inav6 | Inav7) => Some(Self::Dshot150),
            (5u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4) => Some(Self::Dshot150),
            (5u32, Inav5 | Inav6 | Inav7) => Some(Self::Dshot300),
            (6u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4) => Some(Self::Dshot300),
            (6u32, Inav5 | Inav6 | Inav7) => Some(Self::Dshot600),
            (7u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4) => Some(Self::Dshot600),
            (8u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4) => Some(Self::Proshot1000),
            (9u32, Betaflight4_2 | Betaflight4_3 | Betaflight4_4) => Some(Self::Disabled),
            _ => {
                #[allow(clippy::redundant_closure_call)]
                (|raw| tracing::error!("invalid pwm protocol: {raw}"))(raw);
                None
            }
        }
    }
}
