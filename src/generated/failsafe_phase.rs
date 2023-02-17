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
    unused_qualifications,
    clippy::match_same_arms,
    clippy::unseparated_literal_suffix
)]
impl FailsafePhase {
    pub(crate) fn new(raw: u32, fw: crate::headers::InternalFirmware) -> Self {
        match raw {
            0u32 => Self::Idle,
            1u32 => Self::RxLossDetected,
            2u32 if fw.is_betaflight() => Self::Landing,
            2u32 if fw.is_inav() => Self::RxLossIdle,
            3u32 if fw.is_betaflight() => Self::Landed,
            3u32 if fw.is_inav() => Self::ReturnToHome,
            4u32 if fw.is_betaflight() => Self::RxLossMonitoring,
            4u32 if fw.is_inav() => Self::Landing,
            5u32 if fw.is_betaflight() => Self::RxLossRecovered,
            5u32 if fw.is_inav() => Self::Landed,
            6u32 if fw.is_betaflight() => Self::GpsRescue,
            6u32 if fw.is_inav() => Self::RxLossMonitoring,
            7u32 if fw.is_inav() => Self::RxLossRecovered,
            _ => {
                #[allow(clippy::redundant_closure_call)]
                (|raw| tracing::debug!("invalid failsafe phase ({raw})"))(raw);
                Self::Unknown
            }
        }
    }
}
