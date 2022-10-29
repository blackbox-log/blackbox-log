use alloc::vec::Vec;
use core::f64;

use crate::common::{FirmwareKind, FlightModeFlags};
use crate::parser::headers::CurrentMeterConfig;
use crate::parser::{as_signed, Headers};
use crate::{betaflight, inav};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Amperage {
    raw: i32,
    current_meter: CurrentMeterConfig,
}

impl Amperage {
    pub const fn new(raw: u32, headers: &Headers) -> Self {
        Self {
            raw: as_signed(raw),
            current_meter: headers.current_meter,
        }
    }

    pub const fn as_raw(&self) -> i32 {
        self.raw
    }

    pub fn as_milliamps(&self) -> f64 {
        let milliamps = f64::from(self.raw * 3300) / 4095.;
        let milliamps = milliamps - f64::from(self.current_meter.offset);
        (milliamps * 10_000.) / f64::from(self.current_meter.scale)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Voltage {
    pub(crate) raw: u32,
    pub(crate) scale: u16,
}

impl Voltage {
    pub const fn new(raw: u32, headers: &Headers) -> Self {
        Self {
            raw,
            scale: headers.vbat_scale,
        }
    }

    pub const fn as_raw(&self) -> u32 {
        self.raw
    }

    pub fn as_millivolts(&self) -> f64 {
        f64::from(self.raw * 330 * u32::from(self.scale)) / 4095.
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Acceleration {
    raw: i32,
    one_g: u16,
}

impl Acceleration {
    pub(crate) fn new(raw: u32, headers: &Headers) -> Self {
        Self {
            raw: as_signed(raw),
            one_g: headers.acceleration_1g,
        }
    }

    pub const fn as_raw(&self) -> i32 {
        self.raw
    }

    pub fn as_gs(&self) -> f64 {
        f64::from(self.raw) / f64::from(self.one_g)
    }

    pub fn as_meters_per_sec_sq(&self) -> f64 {
        self.as_gs() * 9.80665
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rotation(i32);

impl Rotation {
    pub const fn new(raw: u32) -> Self {
        Self(as_signed(raw))
    }

    pub const fn as_raw(&self) -> i32 {
        self.0
    }

    pub fn as_degrees(&self) -> f64 {
        self.0.into()
    }

    pub fn as_radians(&self) -> f64 {
        self.as_degrees() * f64::consts::PI / 180.
    }
}

pub trait FlagSet {
    fn as_raw(&self) -> u32;
    fn as_names(&self) -> Vec<&'static str>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlightMode {
    Betaflight(FlightModeFlags<betaflight::FlightMode>),
    Inav(FlightModeFlags<inav::FlightMode>),
}

impl FlightMode {
    pub fn new(mode: u32, headers: &Headers) -> Self {
        match headers.firmware_kind {
            FirmwareKind::Betaflight => Self::Betaflight(FlightModeFlags::new(mode)),
            FirmwareKind::INav => Self::Inav(FlightModeFlags::new(mode)),
        }
    }
}

impl FlagSet for FlightMode {
    fn as_raw(&self) -> u32 {
        match self {
            Self::Betaflight(m) => m.as_raw(),
            Self::Inav(m) => m.as_raw(),
        }
    }

    fn as_names(&self) -> Vec<&'static str> {
        match self {
            Self::Betaflight(m) => m
                .to_modes()
                .into_iter()
                .map(betaflight::FlightMode::as_name)
                .collect(),
            Self::Inav(m) => m
                .to_modes()
                .into_iter()
                .map(inav::FlightMode::as_name)
                .collect(),
        }
    }
}
