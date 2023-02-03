#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(unused_qualifications)]
pub struct DisabledFields {
    firmware: crate::headers::Firmware,
    raw: ::bitvec::array::BitArray<[u32; 1], ::bitvec::order::Lsb0>,
}
#[allow(unused_qualifications)]
impl DisabledFields {
    pub(crate) fn new(raw: u32, firmware: crate::headers::Firmware) -> Self {
        Self {
            firmware,
            raw: ::bitvec::array::BitArray::new([raw]),
        }
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
        self.raw
            .iter_ones()
            .filter_map(|bit| {
                let flag = <FieldGroup>::from_bit(bit as u32, self.firmware)?;
                let name = <FieldGroup as crate::units::Flag>::as_name(&flag);
                Some(name)
            })
            .collect()
    }
}
impl ::core::fmt::Display for DisabledFields {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let names = <Self as crate::units::FlagSet>::as_names(self);
        f.write_str(&names.join("|"))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    unused_imports,
    unused_qualifications,
    clippy::match_same_arms,
    clippy::unseparated_literal_suffix
)]
impl FieldGroup {
    const fn from_bit(bit: u32, firmware: crate::headers::Firmware) -> Option<Self> {
        use crate::headers::Firmware::{Betaflight, Inav};
        match (bit, firmware) {
            (0u32, Betaflight(_)) => Some(Self::Pid),
            (1u32, Betaflight(_)) => Some(Self::RcCommands),
            (2u32, Betaflight(_)) => Some(Self::Setpoint),
            (3u32, Betaflight(_)) => Some(Self::Battery),
            (4u32, Betaflight(_)) => Some(Self::Mag),
            (5u32, Betaflight(_)) => Some(Self::Altitude),
            (6u32, Betaflight(_)) => Some(Self::Rssi),
            (7u32, Betaflight(_)) => Some(Self::Gyro),
            (8u32, Betaflight(_)) => Some(Self::Acc),
            (9u32, Betaflight(_)) => Some(Self::DebugLog),
            (10u32, Betaflight(_)) => Some(Self::Motor),
            (11u32, Betaflight(_)) => Some(Self::Gps),
            _ => None,
        }
    }

    const fn to_bit(self, firmware: crate::headers::Firmware) -> Option<u32> {
        use crate::headers::Firmware::{Betaflight, Inav};
        match (self, firmware) {
            (Self::Acc, Betaflight(_)) => Some(8u32),
            (Self::Altitude, Betaflight(_)) => Some(5u32),
            (Self::Battery, Betaflight(_)) => Some(3u32),
            (Self::DebugLog, Betaflight(_)) => Some(9u32),
            (Self::Gps, Betaflight(_)) => Some(11u32),
            (Self::Gyro, Betaflight(_)) => Some(7u32),
            (Self::Mag, Betaflight(_)) => Some(4u32),
            (Self::Motor, Betaflight(_)) => Some(10u32),
            (Self::Pid, Betaflight(_)) => Some(0u32),
            (Self::RcCommands, Betaflight(_)) => Some(1u32),
            (Self::Rssi, Betaflight(_)) => Some(6u32),
            (Self::Setpoint, Betaflight(_)) => Some(2u32),
            _ => None,
        }
    }
}
