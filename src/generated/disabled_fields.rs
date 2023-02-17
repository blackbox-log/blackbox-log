#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(unused_qualifications)]
pub struct DisabledFields {
    firmware: crate::headers::InternalFirmware,
    raw: ::bitvec::array::BitArray<u32, ::bitvec::order::Lsb0>,
}
#[allow(unused_qualifications, clippy::cast_possible_truncation)]
impl DisabledFields {
    pub(crate) fn new(raw: u32, firmware: crate::headers::InternalFirmware) -> Self {
        Self {
            firmware,
            raw: ::bitvec::array::BitArray::new(raw),
        }
    }

    fn iter(&self) -> impl Iterator<Item = <Self as crate::units::FlagSet>::Flag> + '_ {
        self.raw
            .iter_ones()
            .filter_map(|bit| <FieldGroup>::from_bit(bit as u32, self.firmware))
    }
}
#[allow(unused_qualifications, clippy::cast_possible_truncation)]
impl crate::units::FlagSet for DisabledFields {
    type Flag = FieldGroup;

    fn is_set(&self, flag: Self::Flag) -> bool {
        flag.to_bit(self.firmware)
            .map_or(false, |bit| self.raw[bit as usize])
    }

    fn as_names(&self) -> ::alloc::vec::Vec<&'static str> {
        self.iter()
            .map(|flag| <FieldGroup as crate::units::Flag>::as_name(&flag))
            .collect()
    }
}
impl ::core::fmt::Display for DisabledFields {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let names = <Self as crate::units::FlagSet>::as_names(self);
        f.write_str(&names.join("|"))
    }
}
#[cfg(feature = "_serde")]
#[allow(clippy::cast_possible_truncation)]
impl ::serde::Serialize for DisabledFields {
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
pub enum FieldGroup {
    /// `ACC` (Betaflight only)
    Acc,
    /// `ALTITUDE` (Betaflight only)
    Altitude,
    /// `BATTERY` (Betaflight only)
    Battery,
    /// `DEBUG_LOG` (Betaflight only)
    DebugLog,
    /// `GPS` (Betaflight only)
    Gps,
    /// `GYRO` (Betaflight only)
    Gyro,
    /// `MAG` (Betaflight only)
    Mag,
    /// `MOTOR` (Betaflight only)
    Motor,
    /// `PID` (Betaflight only)
    Pid,
    /// `RC_COMMANDS` (Betaflight only)
    RcCommands,
    /// `RSSI` (Betaflight only)
    Rssi,
    /// `SETPOINT` (Betaflight only)
    Setpoint,
}
#[allow(unused_qualifications)]
impl crate::units::Flag for FieldGroup {
    fn as_name(&self) -> &'static str {
        match self {
            Self::Acc => "ACC",
            Self::Altitude => "ALTITUDE",
            Self::Battery => "BATTERY",
            Self::DebugLog => "DEBUG_LOG",
            Self::Gps => "GPS",
            Self::Gyro => "GYRO",
            Self::Mag => "MAG",
            Self::Motor => "MOTOR",
            Self::Pid => "PID",
            Self::RcCommands => "RC_COMMANDS",
            Self::Rssi => "RSSI",
            Self::Setpoint => "SETPOINT",
        }
    }
}
impl ::core::fmt::Display for FieldGroup {
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
impl FieldGroup {
    const fn from_bit(bit: u32, fw: crate::headers::InternalFirmware) -> Option<Self> {
        match bit {
            0u32 if fw.is_betaflight() => Some(Self::Pid),
            1u32 if fw.is_betaflight() => Some(Self::RcCommands),
            2u32 if fw.is_betaflight() => Some(Self::Setpoint),
            3u32 if fw.is_betaflight() => Some(Self::Battery),
            4u32 if fw.is_betaflight() => Some(Self::Mag),
            5u32 if fw.is_betaflight() => Some(Self::Altitude),
            6u32 if fw.is_betaflight() => Some(Self::Rssi),
            7u32 if fw.is_betaflight() => Some(Self::Gyro),
            8u32 if fw.is_betaflight() => Some(Self::Acc),
            9u32 if fw.is_betaflight() => Some(Self::DebugLog),
            10u32 if fw.is_betaflight() => Some(Self::Motor),
            11u32 if fw.is_betaflight() => Some(Self::Gps),
            _ => None,
        }
    }

    const fn to_bit(self, fw: crate::headers::InternalFirmware) -> Option<u32> {
        match self {
            Self::Acc if fw.is_betaflight() => Some(8u32),
            Self::Altitude if fw.is_betaflight() => Some(5u32),
            Self::Battery if fw.is_betaflight() => Some(3u32),
            Self::DebugLog if fw.is_betaflight() => Some(9u32),
            Self::Gps if fw.is_betaflight() => Some(11u32),
            Self::Gyro if fw.is_betaflight() => Some(7u32),
            Self::Mag if fw.is_betaflight() => Some(4u32),
            Self::Motor if fw.is_betaflight() => Some(10u32),
            Self::Pid if fw.is_betaflight() => Some(0u32),
            Self::RcCommands if fw.is_betaflight() => Some(1u32),
            Self::Rssi if fw.is_betaflight() => Some(6u32),
            Self::Setpoint if fw.is_betaflight() => Some(2u32),
            _ => None,
        }
    }
}
