#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(unused_qualifications)]
pub struct FeatureSet {
    firmware: crate::headers::InternalFirmware,
    raw: ::bitvec::array::BitArray<u32, ::bitvec::order::Lsb0>,
}
#[allow(unused_qualifications, clippy::cast_possible_truncation)]
impl FeatureSet {
    pub(crate) fn new(raw: u32, firmware: crate::headers::InternalFirmware) -> Self {
        Self {
            firmware,
            raw: ::bitvec::array::BitArray::new(raw),
        }
    }

    fn iter(&self) -> impl Iterator<Item = <Self as crate::units::FlagSet>::Flag> + '_ {
        self.raw
            .iter_ones()
            .filter_map(|bit| <Feature>::from_bit(bit as u32, self.firmware))
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
        self.iter()
            .map(|flag| <Feature as crate::units::Flag>::as_name(&flag))
            .collect()
    }
}
impl ::core::fmt::Display for FeatureSet {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let names = <Self as crate::units::FlagSet>::as_names(self);
        f.write_str(&names.join("|"))
    }
}
#[cfg(feature = "_serde")]
#[allow(clippy::cast_possible_truncation)]
impl ::serde::Serialize for FeatureSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(None)?;
        for flag in self.iter() {
            seq.serialize_element(&flag)?;
        }
        seq.end()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "_serde", derive(serde::Serialize))]
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
        let s = <Self as crate::units::Flag>::as_name(self);
        f.write_str(s)
    }
}
#[allow(
    unused_qualifications,
    clippy::match_same_arms,
    clippy::unseparated_literal_suffix,
    clippy::wildcard_enum_match_arm
)]
impl Feature {
    const fn from_bit(bit: u32, fw: crate::headers::InternalFirmware) -> Option<Self> {
        match bit {
            0u32 if fw.is_betaflight() => Some(Self::RxPpm),
            0u32 if fw.is_inav() => Some(Self::ThrottleVbatCompensation),
            1u32 if fw.is_inav() => Some(Self::Vbat),
            2u32 if fw.is_betaflight() => Some(Self::InflightAccCal),
            2u32 if fw.is_inav() => Some(Self::TxProfileSelection),
            3u32 if fw.is_betaflight() => Some(Self::RxSerial),
            3u32 if fw.is_inav() => Some(Self::BatProfileAutoswitch),
            4u32 => Some(Self::MotorStop),
            5u32 if fw.is_betaflight() => Some(Self::ServoTilt),
            6u32 => Some(Self::SoftSerial),
            7u32 => Some(Self::Gps),
            9u32 if fw.is_betaflight() => Some(Self::RangeFinder),
            10u32 => Some(Self::Telemetry),
            11u32 if fw.is_inav() => Some(Self::CurrentMeter),
            12u32 if fw.is_betaflight() => Some(Self::ThreeD),
            12u32 if fw.is_inav() => Some(Self::ReversibleMotors),
            13u32 if fw.is_betaflight() => Some(Self::RxParallelPwm),
            14u32 if fw.is_betaflight() => Some(Self::RxMsp),
            15u32 => Some(Self::RssiAdc),
            16u32 => Some(Self::LedStrip),
            17u32 => Some(Self::Dashboard),
            18u32 if fw.is_betaflight() => Some(Self::Osd),
            19u32 if fw.is_inav() => Some(Self::Blackbox),
            20u32 if fw.is_betaflight() => Some(Self::ChannelForwarding),
            21u32 => Some(Self::Transponder),
            22u32 => Some(Self::AirMode),
            23u32 if fw.is_inav() => Some(Self::SuperexpoRates),
            24u32 if fw.is_inav() => Some(Self::Vtx),
            25u32 if fw.is_betaflight() => Some(Self::RxSpi),
            27u32 if fw.is_betaflight() => Some(Self::EscSensor),
            28u32 if fw.is_betaflight() => Some(Self::AntiGravity),
            28u32 if fw.is_inav() => Some(Self::PwmOutputEnable),
            29u32 if fw.is_inav() => Some(Self::Osd),
            30u32 if fw.is_inav() => Some(Self::FwLaunch),
            31u32 if fw.is_inav() => Some(Self::FwAutotrim),
            _ => None,
        }
    }

    const fn to_bit(self, fw: crate::headers::InternalFirmware) -> Option<u32> {
        match self {
            Self::AirMode => Some(22u32),
            Self::AntiGravity if fw.is_betaflight() => Some(28u32),
            Self::BatProfileAutoswitch if fw.is_inav() => Some(3u32),
            Self::Blackbox if fw.is_inav() => Some(19u32),
            Self::ChannelForwarding if fw.is_betaflight() => Some(20u32),
            Self::CurrentMeter if fw.is_inav() => Some(11u32),
            Self::Dashboard => Some(17u32),
            Self::EscSensor if fw.is_betaflight() => Some(27u32),
            Self::FwAutotrim if fw.is_inav() => Some(31u32),
            Self::FwLaunch if fw.is_inav() => Some(30u32),
            Self::Gps => Some(7u32),
            Self::InflightAccCal if fw.is_betaflight() => Some(2u32),
            Self::LedStrip => Some(16u32),
            Self::MotorStop => Some(4u32),
            Self::Osd if fw.is_betaflight() => Some(18u32),
            Self::Osd if fw.is_inav() => Some(29u32),
            Self::PwmOutputEnable if fw.is_inav() => Some(28u32),
            Self::RangeFinder if fw.is_betaflight() => Some(9u32),
            Self::ReversibleMotors if fw.is_inav() => Some(12u32),
            Self::RssiAdc => Some(15u32),
            Self::RxMsp if fw.is_betaflight() => Some(14u32),
            Self::RxParallelPwm if fw.is_betaflight() => Some(13u32),
            Self::RxPpm if fw.is_betaflight() => Some(0u32),
            Self::RxSerial if fw.is_betaflight() => Some(3u32),
            Self::RxSpi if fw.is_betaflight() => Some(25u32),
            Self::ServoTilt if fw.is_betaflight() => Some(5u32),
            Self::SoftSerial => Some(6u32),
            Self::SuperexpoRates if fw.is_inav() => Some(23u32),
            Self::Telemetry => Some(10u32),
            Self::ThreeD if fw.is_betaflight() => Some(12u32),
            Self::ThrottleVbatCompensation if fw.is_inav() => Some(0u32),
            Self::Transponder => Some(21u32),
            Self::TxProfileSelection if fw.is_inav() => Some(2u32),
            Self::Vbat if fw.is_inav() => Some(1u32),
            Self::Vtx if fw.is_inav() => Some(24u32),
            _ => None,
        }
    }
}
