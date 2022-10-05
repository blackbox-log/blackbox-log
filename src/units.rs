use core::f64;

use crate::parser::headers::CurrentMeterConfig;
use crate::parser::Headers;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Acceleration(Acceleration),
    Amperage(Amperage),
    FrameTime(i64),
    Rotation(Rotation),
    Voltage(Voltage),
    Unitless(i64),
}

impl Unit {
    pub const fn as_raw(self) -> i64 {
        match self {
            Self::Acceleration(a) => a.as_raw(),
            Self::Amperage(a) => a.as_raw(),
            Self::FrameTime(t) => t,
            Self::Rotation(r) => r.as_raw(),
            Self::Voltage(v) => v.as_raw(),
            Self::Unitless(x) => x,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitKind {
    Acceleration,
    Amperage,
    FrameTime,
    Rotation,
    Voltage,
    Unitless,
}

impl UnitKind {
    pub(crate) fn with_value(self, value: i64, headers: &Headers) -> Unit {
        match self {
            Self::Acceleration => Unit::Acceleration(Acceleration {
                raw: value,
                one_g: headers.acceleration_1g,
            }),
            Self::Amperage => Unit::Amperage(Amperage {
                raw: value,
                current_meter: headers.current_meter,
            }),
            Self::FrameTime => Unit::FrameTime(value),
            Self::Rotation => Unit::Rotation(Rotation(value)),
            Self::Voltage => Unit::Voltage(Voltage {
                raw: value,
                scale: headers.vbat_scale,
            }),
            Self::Unitless => Unit::Unitless(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Acceleration {
    raw: i64,
    one_g: u16,
}

impl Acceleration {
    pub const fn as_raw(&self) -> i64 {
        self.raw
    }

    pub fn as_gs(&self) -> f64 {
        i64_to_f64(self.raw) / f64::from(self.one_g)
    }

    pub fn as_meters_per_sec_sq(&self) -> f64 {
        self.as_gs() * 9.80665
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amperage {
    raw: i64,
    current_meter: CurrentMeterConfig,
}

impl Amperage {
    pub const fn as_raw(&self) -> i64 {
        self.raw
    }

    pub fn as_milliamps(&self) -> f64 {
        let milliamps = i64_to_f64(self.raw * 3300) / 4095.;
        let milliamps = milliamps - f64::from(self.current_meter.offset);
        (milliamps * 10_000.) / f64::from(self.current_meter.scale)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rotation(i64);

impl Rotation {
    pub const fn as_raw(&self) -> i64 {
        self.0
    }

    pub fn as_degrees(&self) -> f64 {
        self.0 as f64
    }

    pub fn as_radians(&self) -> f64 {
        self.as_degrees() * f64::consts::PI / 180.
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Voltage {
    pub(crate) raw: i64,
    pub(crate) scale: u16,
}

impl Voltage {
    pub const fn as_raw(&self) -> i64 {
        self.raw
    }

    pub fn as_millivolts(&self) -> f64 {
        i64_to_f64(self.raw * 330 * i64::from(self.scale)) / 4095.
    }
}

#[inline]
fn i64_to_f64(x: i64) -> f64 {
    f64::from(i32::try_from(x).unwrap())
}
