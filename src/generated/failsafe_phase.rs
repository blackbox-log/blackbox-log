#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "_serde", derive(serde::Serialize))]
/// The current failsafe phase. See [`Flag`][crate::units::Flag].
pub enum FailsafePhase {
    /// `GPS_RESCUE` (Betaflight only)
    GpsRescue,
    /// `IDLE`
    Idle,
    /// `LANDED`
    Landed,
    /// `LANDING`
    Landing,
    /// `RETURN_TO_HOME` (INAV only)
    ReturnToHome,
    /// `RX_LOSS_DETECTED`
    RxLossDetected,
    /// `RX_LOSS_IDLE` (INAV only)
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
impl ::core::fmt::Display for FailsafePhase {
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
impl FailsafePhase {
    pub(crate) fn new(raw: u32, firmware: crate::headers::Firmware) -> Self {
        use crate::headers::Firmware::{Betaflight, Inav};
        match (raw, firmware) {
            (0u32, _) => Self::Idle,
            (1u32, _) => Self::RxLossDetected,
            (2u32, Betaflight(_)) => Self::Landing,
            (2u32, Inav(_)) => Self::RxLossIdle,
            (3u32, Betaflight(_)) => Self::Landed,
            (3u32, Inav(_)) => Self::ReturnToHome,
            (4u32, Betaflight(_)) => Self::RxLossMonitoring,
            (4u32, Inav(_)) => Self::Landing,
            (5u32, Betaflight(_)) => Self::RxLossRecovered,
            (5u32, Inav(_)) => Self::Landed,
            (6u32, Betaflight(_)) => Self::GpsRescue,
            (6u32, Inav(_)) => Self::RxLossMonitoring,
            (7u32, Inav(_)) => Self::RxLossRecovered,
            _ => {
                #[allow(clippy::redundant_closure_call)]
                (|raw| tracing::debug!("invalid failsafe phase ({raw})"))(raw);
                Self::Unknown
            }
        }
    }
}
