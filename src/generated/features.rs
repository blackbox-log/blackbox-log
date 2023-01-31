#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(unused_qualifications)]
pub struct FeatureSet {
    firmware: crate::headers::FirmwareKind,
    raw: ::bitvec::array::BitArray<[u32; 1], ::bitvec::order::Lsb0>,
}
#[allow(unused_qualifications)]
impl FeatureSet {
    pub(crate) fn new(raw: u32, firmware: crate::headers::FirmwareKind) -> Self {
        Self {
            firmware,
            raw: ::bitvec::array::BitArray::new([raw]),
        }
    }
}
#[allow(unused_qualifications, clippy::cast_possible_truncation)]
impl crate::units::FlagSet for FeatureSet {
    type Flag = Feature;

    fn is_set(&self, flag: Self::Flag) -> bool {
        flag.to_bit(self.firmware)
            .map_or(false, |bit| self.raw[bit as usize])
    }

    fn as_names(&self) -> ::alloc::vec::Vec<&'static str> {
        self.raw
            .iter_ones()
            .filter_map(|bit| Some(<Feature>::from_bit(bit as u32, self.firmware)?.as_name()))
            .collect()
    }
}
impl ::core::fmt::Display for FeatureSet {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.write_str(&self.as_names().join("|"))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Feature {
    /// `AIRMODE`
    AirMode,
    /// `ANTI_GRAVITY` (Betaflight only)
    AntiGravity,
    /// `BAT_PROFILE_AUTOSWITCH` (INAV only)
    BatProfileAutoswitch,
    /// `BLACKBOX` (INAV only)
    Blackbox,
    /// `CHANNEL_FORWARDING` (Betaflight only)
    ChannelForwarding,
    /// `CURRENT_METER` (INAV only)
    CurrentMeter,
    /// `DASHBOARD`
    Dashboard,
    /// `ESC_SENSOR` (Betaflight only)
    EscSensor,
    /// `FW_AUTOTRIM` (INAV only)
    FwAutotrim,
    /// `FW_LAUNCH` (INAV only)
    FwLaunch,
    /// `GPS`
    Gps,
    /// `INFLIGHT_ACC_CAL` (Betaflight only)
    InflightAccCal,
    /// `LED_STRIP`
    LedStrip,
    /// `MOTOR_STOP`
    MotorStop,
    /// `OSD`
    Osd,
    /// `PWM_OUTPUT_ENABLE` (INAV only)
    PwmOutputEnable,
    /// `RANGEFINDER` (Betaflight only)
    RangeFinder,
    /// `REVERSIBLE_MOTORS` (INAV only)
    ReversibleMotors,
    /// `RSSI_ADC`
    RssiAdc,
    /// `RX_MSP` (Betaflight only)
    RxMsp,
    /// `RX_PARALLEL_PWM` (Betaflight only)
    RxParallelPwm,
    /// `RX_PPM` (Betaflight only)
    RxPpm,
    /// `RX_SERIAL` (Betaflight only)
    RxSerial,
    /// `RX_SPI` (Betaflight only)
    RxSpi,
    /// `SERVO_TILT` (Betaflight only)
    ServoTilt,
    /// `SOFTSERIAL`
    SoftSerial,
    /// `SUPEREXPO_RATES` (INAV only)
    SuperexpoRates,
    /// `TELEMETRY`
    Telemetry,
    /// `3D` (Betaflight only)
    ThreeD,
    /// `THR_VBAT_COMP` (INAV only)
    ThrottleVbatCompensation,
    /// `TRANSPONDER`
    Transponder,
    /// `TX_PROF_SEL` (INAV only)
    TxProfileSelection,
    /// `VBAT` (INAV only)
    Vbat,
    /// `VTX` (INAV only)
    Vtx,
}
#[allow(unused_qualifications)]
impl crate::units::Flag for Feature {
    fn as_name(&self) -> &'static str {
        match self {
            Self::AirMode => "AIRMODE",
            Self::AntiGravity => "ANTI_GRAVITY",
            Self::BatProfileAutoswitch => "BAT_PROFILE_AUTOSWITCH",
            Self::Blackbox => "BLACKBOX",
            Self::ChannelForwarding => "CHANNEL_FORWARDING",
            Self::CurrentMeter => "CURRENT_METER",
            Self::Dashboard => "DASHBOARD",
            Self::EscSensor => "ESC_SENSOR",
            Self::FwAutotrim => "FW_AUTOTRIM",
            Self::FwLaunch => "FW_LAUNCH",
            Self::Gps => "GPS",
            Self::InflightAccCal => "INFLIGHT_ACC_CAL",
            Self::LedStrip => "LED_STRIP",
            Self::MotorStop => "MOTOR_STOP",
            Self::Osd => "OSD",
            Self::PwmOutputEnable => "PWM_OUTPUT_ENABLE",
            Self::RangeFinder => "RANGEFINDER",
            Self::ReversibleMotors => "REVERSIBLE_MOTORS",
            Self::RssiAdc => "RSSI_ADC",
            Self::RxMsp => "RX_MSP",
            Self::RxParallelPwm => "RX_PARALLEL_PWM",
            Self::RxPpm => "RX_PPM",
            Self::RxSerial => "RX_SERIAL",
            Self::RxSpi => "RX_SPI",
            Self::ServoTilt => "SERVO_TILT",
            Self::SoftSerial => "SOFTSERIAL",
            Self::SuperexpoRates => "SUPEREXPO_RATES",
            Self::Telemetry => "TELEMETRY",
            Self::ThreeD => "3D",
            Self::ThrottleVbatCompensation => "THR_VBAT_COMP",
            Self::Transponder => "TRANSPONDER",
            Self::TxProfileSelection => "TX_PROF_SEL",
            Self::Vbat => "VBAT",
            Self::Vtx => "VTX",
        }
    }
}
impl ::core::fmt::Display for Feature {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.write_str(self.as_name())
    }
}
#[allow(
    unused_qualifications,
    clippy::match_same_arms,
    clippy::unseparated_literal_suffix
)]
impl Feature {
    const fn from_bit(bit: u32, firmware: crate::headers::FirmwareKind) -> Option<Self> {
        use crate::headers::FirmwareKind::{Betaflight, EmuFlight, Inav};
        match (bit, firmware) {
            (0u32, Betaflight | EmuFlight) => Some(Self::RxPpm),
            (0u32, Inav) => Some(Self::ThrottleVbatCompensation),
            (1u32, Inav) => Some(Self::Vbat),
            (2u32, Betaflight | EmuFlight) => Some(Self::InflightAccCal),
            (2u32, Inav) => Some(Self::TxProfileSelection),
            (3u32, Betaflight | EmuFlight) => Some(Self::RxSerial),
            (3u32, Inav) => Some(Self::BatProfileAutoswitch),
            (4u32, _) => Some(Self::MotorStop),
            (5u32, Betaflight | EmuFlight) => Some(Self::ServoTilt),
            (6u32, _) => Some(Self::SoftSerial),
            (7u32, _) => Some(Self::Gps),
            (9u32, Betaflight | EmuFlight) => Some(Self::RangeFinder),
            (10u32, _) => Some(Self::Telemetry),
            (11u32, Inav) => Some(Self::CurrentMeter),
            (12u32, Betaflight | EmuFlight) => Some(Self::ThreeD),
            (12u32, Inav) => Some(Self::ReversibleMotors),
            (13u32, Betaflight | EmuFlight) => Some(Self::RxParallelPwm),
            (14u32, Betaflight | EmuFlight) => Some(Self::RxMsp),
            (15u32, _) => Some(Self::RssiAdc),
            (16u32, _) => Some(Self::LedStrip),
            (17u32, _) => Some(Self::Dashboard),
            (18u32, Betaflight | EmuFlight) => Some(Self::Osd),
            (19u32, Inav) => Some(Self::Blackbox),
            (20u32, Betaflight | EmuFlight) => Some(Self::ChannelForwarding),
            (21u32, _) => Some(Self::Transponder),
            (22u32, _) => Some(Self::AirMode),
            (23u32, Inav) => Some(Self::SuperexpoRates),
            (24u32, Inav) => Some(Self::Vtx),
            (25u32, Betaflight | EmuFlight) => Some(Self::RxSpi),
            (27u32, Betaflight | EmuFlight) => Some(Self::EscSensor),
            (28u32, Betaflight | EmuFlight) => Some(Self::AntiGravity),
            (28u32, Inav) => Some(Self::PwmOutputEnable),
            (29u32, Inav) => Some(Self::Osd),
            (30u32, Inav) => Some(Self::FwLaunch),
            (31u32, Inav) => Some(Self::FwAutotrim),
            _ => None,
        }
    }

    const fn to_bit(self, firmware: crate::headers::FirmwareKind) -> Option<u32> {
        use crate::headers::FirmwareKind::{Betaflight, EmuFlight, Inav};
        match (self, firmware) {
            (Self::AirMode, _) => Some(22u32),
            (Self::AntiGravity, Betaflight | EmuFlight) => Some(28u32),
            (Self::BatProfileAutoswitch, Inav) => Some(3u32),
            (Self::Blackbox, Inav) => Some(19u32),
            (Self::ChannelForwarding, Betaflight | EmuFlight) => Some(20u32),
            (Self::CurrentMeter, Inav) => Some(11u32),
            (Self::Dashboard, _) => Some(17u32),
            (Self::EscSensor, Betaflight | EmuFlight) => Some(27u32),
            (Self::FwAutotrim, Inav) => Some(31u32),
            (Self::FwLaunch, Inav) => Some(30u32),
            (Self::Gps, _) => Some(7u32),
            (Self::InflightAccCal, Betaflight | EmuFlight) => Some(2u32),
            (Self::LedStrip, _) => Some(16u32),
            (Self::MotorStop, _) => Some(4u32),
            (Self::Osd, Betaflight | EmuFlight) => Some(18u32),
            (Self::Osd, Inav) => Some(29u32),
            (Self::PwmOutputEnable, Inav) => Some(28u32),
            (Self::RangeFinder, Betaflight | EmuFlight) => Some(9u32),
            (Self::ReversibleMotors, Inav) => Some(12u32),
            (Self::RssiAdc, _) => Some(15u32),
            (Self::RxMsp, Betaflight | EmuFlight) => Some(14u32),
            (Self::RxParallelPwm, Betaflight | EmuFlight) => Some(13u32),
            (Self::RxPpm, Betaflight | EmuFlight) => Some(0u32),
            (Self::RxSerial, Betaflight | EmuFlight) => Some(3u32),
            (Self::RxSpi, Betaflight | EmuFlight) => Some(25u32),
            (Self::ServoTilt, Betaflight | EmuFlight) => Some(5u32),
            (Self::SoftSerial, _) => Some(6u32),
            (Self::SuperexpoRates, Inav) => Some(23u32),
            (Self::Telemetry, _) => Some(10u32),
            (Self::ThreeD, Betaflight | EmuFlight) => Some(12u32),
            (Self::ThrottleVbatCompensation, Inav) => Some(0u32),
            (Self::Transponder, _) => Some(21u32),
            (Self::TxProfileSelection, Inav) => Some(2u32),
            (Self::Vbat, Inav) => Some(1u32),
            (Self::Vtx, Inav) => Some(24u32),
            _ => None,
        }
    }
}
