#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// All enabled features.
///
/// See [`FlagSet`][crate::units::FlagSet] and [`Feature`].
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
/// A feature defined by any supported firmware.
///
/// See [`Flag`][crate::units::Flag].
#[non_exhaustive]
pub enum Feature {
    /// `AIRMODE`
    AirMode,
    /// `ANTI_GRAVITY`
    AntiGravity,
    /// `BAT_PROFILE_AUTOSWITCH`
    BatProfileAutoswitch,
    /// `BLACKBOX`
    Blackbox,
    /// `CHANNEL_FORWARDING`
    ChannelForwarding,
    /// `CURRENT_METER`
    CurrentMeter,
    /// `DASHBOARD`
    Dashboard,
    /// `DYNAMIC_FILTER`
    DynamicFilter,
    /// `ESC_SENSOR`
    EscSensor,
    /// `FW_AUTOTRIM`
    FwAutotrim,
    /// `FW_LAUNCH`
    FwLaunch,
    /// `GPS`
    Gps,
    /// `INFLIGHT_ACC_CAL`
    InflightAccCal,
    /// `LED_STRIP`
    LedStrip,
    /// `MOTOR_STOP`
    MotorStop,
    /// `OSD`
    Osd,
    /// `PWM_OUTPUT_ENABLE`
    PwmOutputEnable,
    /// `RANGEFINDER`
    RangeFinder,
    /// `REVERSIBLE_MOTORS`
    ReversibleMotors,
    /// `RSSI_ADC`
    RssiAdc,
    /// `RX_MSP`
    RxMsp,
    /// `RX_PARALLEL_PWM`
    RxParallelPwm,
    /// `RX_PPM`
    RxPpm,
    /// `RX_SERIAL`
    RxSerial,
    /// `RX_SPI`
    RxSpi,
    /// `SERVO_TILT`
    ServoTilt,
    /// `SOFTSERIAL`
    SoftSerial,
    /// `SUPEREXPO_RATES`
    SuperexpoRates,
    /// `TELEMETRY`
    Telemetry,
    /// `3D`
    ThreeD,
    /// `THR_VBAT_COMP`
    ThrottleVbatCompensation,
    /// `TRANSPONDER`
    Transponder,
    /// `TX_PROF_SEL`
    TxProfileSelection,
    /// `VBAT`
    Vbat,
    /// `VTX`
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
            Self::DynamicFilter => "DYNAMIC_FILTER",
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
    clippy::enum_glob_use,
    clippy::match_same_arms,
    clippy::unseparated_literal_suffix
)]
impl Feature {
    const fn from_bit(bit: u32, fw: crate::headers::InternalFirmware) -> Option<Self> {
        use crate::headers::InternalFirmware::*;
        match (bit, fw) {
            (0u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(Self::RxPpm),
            (0u32, Inav5_0_0) => Some(Self::ThrottleVbatCompensation),
            (1u32, Inav5_0_0) => Some(Self::Vbat),
            (2u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => {
                Some(Self::InflightAccCal)
            }
            (2u32, Inav5_0_0) => Some(Self::TxProfileSelection),
            (3u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(Self::RxSerial),
            (3u32, Inav5_0_0) => Some(Self::BatProfileAutoswitch),
            (4u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(Self::MotorStop)
            }
            (5u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(Self::ServoTilt),
            (6u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(Self::SoftSerial)
            }
            (7u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(Self::Gps)
            }
            (9u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(Self::RangeFinder),
            (10u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(Self::Telemetry)
            }
            (11u32, Inav5_0_0) => Some(Self::CurrentMeter),
            (12u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(Self::ThreeD),
            (12u32, Inav5_0_0) => Some(Self::ReversibleMotors),
            (13u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => {
                Some(Self::RxParallelPwm)
            }
            (14u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(Self::RxMsp),
            (15u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(Self::RssiAdc)
            }
            (16u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(Self::LedStrip)
            }
            (17u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(Self::Dashboard)
            }
            (18u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(Self::Osd),
            (19u32, Inav5_0_0) => Some(Self::Blackbox),
            (20u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => {
                Some(Self::ChannelForwarding)
            }
            (21u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(Self::Transponder)
            }
            (22u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(Self::AirMode)
            }
            (23u32, Inav5_0_0) => Some(Self::SuperexpoRates),
            (24u32, Inav5_0_0) => Some(Self::Vtx),
            (25u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(Self::RxSpi),
            (27u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(Self::EscSensor),
            (28u32, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(Self::AntiGravity),
            (28u32, Inav5_0_0) => Some(Self::PwmOutputEnable),
            (29u32, Betaflight4_2_0) => Some(Self::DynamicFilter),
            (29u32, Inav5_0_0) => Some(Self::Osd),
            (30u32, Inav5_0_0) => Some(Self::FwLaunch),
            (31u32, Inav5_0_0) => Some(Self::FwAutotrim),
            _ => None,
        }
    }

    const fn to_bit(self, fw: crate::headers::InternalFirmware) -> Option<u32> {
        use crate::headers::InternalFirmware::*;
        match (self, fw) {
            (Self::RxPpm, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(0u32),
            (Self::ThrottleVbatCompensation, Inav5_0_0) => Some(0u32),
            (Self::Vbat, Inav5_0_0) => Some(1u32),
            (Self::InflightAccCal, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => {
                Some(2u32)
            }
            (Self::TxProfileSelection, Inav5_0_0) => Some(2u32),
            (Self::RxSerial, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(3u32),
            (Self::BatProfileAutoswitch, Inav5_0_0) => Some(3u32),
            (Self::MotorStop, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(4u32)
            }
            (Self::ServoTilt, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(5u32),
            (Self::SoftSerial, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(6u32)
            }
            (Self::Gps, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(7u32)
            }
            (Self::RangeFinder, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(9u32),
            (Self::Telemetry, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(10u32)
            }
            (Self::CurrentMeter, Inav5_0_0) => Some(11u32),
            (Self::ThreeD, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(12u32),
            (Self::ReversibleMotors, Inav5_0_0) => Some(12u32),
            (Self::RxParallelPwm, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => {
                Some(13u32)
            }
            (Self::RxMsp, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(14u32),
            (Self::RssiAdc, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(15u32)
            }
            (Self::LedStrip, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(16u32)
            }
            (Self::Dashboard, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(17u32)
            }
            (Self::Osd, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(18u32),
            (Self::Blackbox, Inav5_0_0) => Some(19u32),
            (Self::ChannelForwarding, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => {
                Some(20u32)
            }
            (
                Self::Transponder,
                Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0,
            ) => Some(21u32),
            (Self::AirMode, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0 | Inav5_0_0) => {
                Some(22u32)
            }
            (Self::SuperexpoRates, Inav5_0_0) => Some(23u32),
            (Self::Vtx, Inav5_0_0) => Some(24u32),
            (Self::RxSpi, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(25u32),
            (Self::EscSensor, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(27u32),
            (Self::AntiGravity, Betaflight4_2_0 | Betaflight4_3_0 | Betaflight4_4_0) => Some(28u32),
            (Self::PwmOutputEnable, Inav5_0_0) => Some(28u32),
            (Self::DynamicFilter, Betaflight4_2_0) => Some(29u32),
            (Self::Osd, Inav5_0_0) => Some(29u32),
            (Self::FwLaunch, Inav5_0_0) => Some(30u32),
            (Self::FwAutotrim, Inav5_0_0) => Some(31u32),
            _ => None,
        }
    }
}
