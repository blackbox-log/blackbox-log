#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "_serde", derive(serde::Serialize))]
/// The current failsafe phase.
#[non_exhaustive]
pub enum FailsafePhase {
    /// `GPS_RESCUE`
    GpsRescue,
    /// `IDLE`
    Idle,
    /// `LANDED`
    Landed,
    /// `LANDING`
    Landing,
    /// `RETURN_TO_HOME`
    ReturnToHome,
    /// `RX_LOSS_DETECTED`
    RxLossDetected,
    /// `RX_LOSS_IDLE`
    RxLossIdle,
    /// `RX_LOSS_MONITORING`
    RxLossMonitoring,
    /// `RX_LOSS_RECOVERED`
    RxLossRecovered,
    Unknown,
}
#[allow(unused_qualifications)]
impl crate::units::Flag for FailsafePhase {
    fn as_name(&self) -> &'static str {
        match self {
            Self::GpsRescue => "GPS_RESCUE",
            Self::Idle => "IDLE",
            Self::Landed => "LANDED",
            Self::Landing => "LANDING",
            Self::ReturnToHome => "RETURN_TO_HOME",
            Self::RxLossDetected => "RX_LOSS_DETECTED",
            Self::RxLossIdle => "RX_LOSS_IDLE",
            Self::RxLossMonitoring => "RX_LOSS_MONITORING",
            Self::RxLossRecovered => "RX_LOSS_RECOVERED",
            Self::Unknown => "UNKNOWN",
        }
    }
}
#[allow(unused_qualifications)]
impl ::core::fmt::Display for FailsafePhase {
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
impl FailsafePhase {
    pub(crate) fn new(raw: u32, fw: crate::headers::InternalFirmware) -> Self {
        use crate::headers::InternalFirmware::*;
        match (raw, fw) {
            (0u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0 | Inav6_0_0) => {
                Self::Idle
            }
            (1u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0 | Inav6_0_0) => {
                Self::RxLossDetected
            }
            (2u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Self::Landing,
            (2u32, Inav5_0_0 | Inav6_0_0) => Self::RxLossIdle,
            (3u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Self::Landed,
            (3u32, Inav5_0_0 | Inav6_0_0) => Self::ReturnToHome,
            (4u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Self::RxLossMonitoring,
            (4u32, Inav5_0_0 | Inav6_0_0) => Self::Landing,
            (5u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Self::RxLossRecovered,
            (5u32, Inav5_0_0 | Inav6_0_0) => Self::Landed,
            (6u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Self::GpsRescue,
            (6u32, Inav5_0_0 | Inav6_0_0) => Self::RxLossMonitoring,
            (7u32, Inav5_0_0 | Inav6_0_0) => Self::RxLossRecovered,
            _ => {
                #[allow(clippy::redundant_closure_call)]
                (|raw| tracing::debug!("invalid failsafe phase ({raw})"))(raw);
                Self::Unknown
            }
        }
    }
}
