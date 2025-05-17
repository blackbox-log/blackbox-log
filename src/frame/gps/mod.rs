mod def;

use alloc::vec::Vec;

pub use self::def::*;
use super::Unit;
use crate::filter::AppliedFilter;
use crate::units::prelude::*;
use crate::{units, Headers};

/// Data parsed from a GPS frame.
#[derive(Debug, Clone)]
pub struct GpsFrame<'data, 'headers, 'parser> {
    headers: &'headers Headers<'data>,
    raw: RawGpsFrame,
    filter: &'parser AppliedFilter,
}

impl super::seal::Sealed for GpsFrame<'_, '_, '_> {}

impl super::Frame for GpsFrame<'_, '_, '_> {
    type Value = GpsValue;

    #[inline]
    fn len(&self) -> usize {
        self.filter.len()
    }

    fn get_raw(&self, index: usize) -> Option<u32> {
        let index = self.filter.get(index)?;
        Some(self.raw.values[index])
    }

    fn get(&self, index: usize) -> Option<Self::Value> {
        let frame_def = self.headers.gps_frame_def().unwrap();
        let index = self.filter.get(index)?;

        let def = &frame_def.fields[index];
        let raw = self.raw.values[index];

        let value = match def.unit {
            GpsUnit::Coordinate => {
                assert!(def.signed);
                let value = raw.cast_signed();

                GpsValue::Coordinate(f64::from(value) / 10000000.)
            }
            GpsUnit::Altitude => {
                let altitude = if def.signed {
                    raw.cast_signed().into()
                } else {
                    raw.into()
                };

                GpsValue::Altitude(Length::new::<meter>(altitude))
            }
            GpsUnit::Velocity => {
                assert!(!def.signed);
                GpsValue::Velocity(units::new::velocity(raw))
            }
            GpsUnit::Heading => {
                assert!(!def.signed);
                GpsValue::Heading(f64::from(raw) / 10.)
            }
            GpsUnit::Unitless => GpsValue::new_unitless(raw, def.signed),
        };

        Some(value)
    }
}

impl<'data, 'headers, 'parser> GpsFrame<'data, 'headers, 'parser> {
    pub(crate) fn new(
        headers: &'headers Headers<'data>,
        raw: RawGpsFrame,
        filter: &'parser AppliedFilter,
    ) -> Self {
        Self {
            headers,
            raw,
            filter,
        }
    }

    /// Returns the parsed time since power on.
    pub fn time(&self) -> Time {
        units::new::time(self.raw.time)
    }

    /// Returns the raw microsecond counter since power on.
    ///
    /// **Note:** This does not currently handle overflow of the transmitted
    /// 32bit counter.
    pub fn time_raw(&self) -> u64 {
        self.raw.time
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RawGpsFrame {
    pub(crate) time: u64,
    pub(crate) values: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpsValue {
    Coordinate(f64),
    Altitude(Length),
    Velocity(Velocity),
    Heading(f64),
    Unsigned(u32),
    Signed(i32),
}

impl GpsValue {
    const fn new_unitless(value: u32, signed: bool) -> Self {
        if signed {
            Self::Signed(value.cast_signed())
        } else {
            Self::Unsigned(value)
        }
    }
}

impl From<GpsValue> for super::Value {
    fn from(value: GpsValue) -> Self {
        match value {
            GpsValue::Coordinate(c) => Self::GpsCoordinate(c),
            GpsValue::Altitude(a) => Self::Altitude(a),
            GpsValue::Velocity(v) => Self::Velocity(v),
            GpsValue::Heading(h) => Self::GpsHeading(h),
            GpsValue::Unsigned(x) => Self::Unsigned(x),
            GpsValue::Signed(x) => Self::Signed(x),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpsUnit {
    Coordinate,
    Altitude,
    Velocity,
    Heading,
    Unitless,
}

impl From<GpsUnit> for Unit {
    fn from(unit: GpsUnit) -> Self {
        match unit {
            GpsUnit::Coordinate => Self::GpsCoordinate,
            GpsUnit::Altitude => Self::Altitude,
            GpsUnit::Velocity => Self::Velocity,
            GpsUnit::Heading => Self::GpsHeading,
            GpsUnit::Unitless => Self::Unitless,
        }
    }
}
