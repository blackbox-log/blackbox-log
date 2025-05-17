#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// All disabled field groups in a Betaflight log.
///
/// See [`FlagSet`][crate::units::FlagSet] and [`FieldGroup`].
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
            .is_some_and(|bit| self.raw[bit as usize])
    }

    fn as_names(&self) -> ::alloc::vec::Vec<&'static str> {
        self.iter()
            .map(|flag| <FieldGroup as crate::units::Flag>::as_name(&flag))
            .collect()
    }
}
#[allow(unused_qualifications)]
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
        use serde::ser::SerializeSeq as _;
        let mut seq = serializer.serialize_seq(None)?;
        for flag in self.iter() {
            seq.serialize_element(&flag)?;
        }
        seq.end()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "_serde", derive(serde::Serialize))]
/// A set of fields that can be disabled in Betaflight logs.
///
/// See [`Flag`][crate::units::Flag].
#[non_exhaustive]
pub enum FieldGroup {
    /// `ACC`
    Acc,
    /// `ALTITUDE`
    Altitude,
    /// `BATTERY`
    Battery,
    /// `DEBUG_LOG`
    DebugLog,
    /// `GPS`
    Gps,
    /// `GYRO`
    Gyro,
    /// `GYROUNFILT`
    GyroUnfiltered,
    /// `MAG`
    Mag,
    /// `MOTOR`
    Motor,
    /// `PID`
    Pid,
    /// `RC_COMMANDS`
    RcCommands,
    /// `RPM`
    Rpm,
    /// `RSSI`
    Rssi,
    /// `SETPOINT`
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
            Self::GyroUnfiltered => "GYROUNFILT",
            Self::Mag => "MAG",
            Self::Motor => "MOTOR",
            Self::Pid => "PID",
            Self::RcCommands => "RC_COMMANDS",
            Self::Rpm => "RPM",
            Self::Rssi => "RSSI",
            Self::Setpoint => "SETPOINT",
        }
    }
}
#[allow(unused_qualifications)]
impl ::core::fmt::Display for FieldGroup {
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
impl FieldGroup {
    const fn from_bit(bit: u32, fw: crate::headers::InternalFirmware) -> Option<Self> {
        use crate::headers::InternalFirmware::*;
        match (bit, fw) {
            (0u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::Pid),
            (1u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::RcCommands),
            (2u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::Setpoint),
            (3u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::Battery),
            (4u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::Mag),
            (5u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::Altitude),
            (6u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::Rssi),
            (7u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::Gyro),
            (8u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::Acc),
            (9u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::DebugLog),
            (10u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::Motor),
            (11u32, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(Self::Gps),
            (12u32, Betaflight4_5) => Some(Self::Rpm),
            (13u32, Betaflight4_5) => Some(Self::GyroUnfiltered),
            _ => None,
        }
    }

    const fn to_bit(self, fw: crate::headers::InternalFirmware) -> Option<u32> {
        use crate::headers::InternalFirmware::*;
        match (self, fw) {
            (Self::Pid, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(0u32),
            (Self::RcCommands, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(1u32),
            (Self::Setpoint, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(2u32),
            (Self::Battery, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(3u32),
            (Self::Mag, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(4u32),
            (Self::Altitude, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(5u32),
            (Self::Rssi, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(6u32),
            (Self::Gyro, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(7u32),
            (Self::Acc, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(8u32),
            (Self::DebugLog, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(9u32),
            (Self::Motor, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(10u32),
            (Self::Gps, Betaflight4_3 | Betaflight4_4 | Betaflight4_5) => Some(11u32),
            (Self::Rpm, Betaflight4_5) => Some(12u32),
            (Self::GyroUnfiltered, Betaflight4_5) => Some(13u32),
            _ => None,
        }
    }
}
