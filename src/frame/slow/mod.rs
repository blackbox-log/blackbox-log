mod def;

use alloc::vec::Vec;

pub use self::def::*;
use super::Unit;
use crate::filter::AppliedFilter;
use crate::utils::as_i32;
use crate::{units, Headers};

/// Data parsed from a slow frame.
///
/// Slow frames do not include any metadata. If that is desired, use the prior
/// [`MainFrame`][super::MainFrame].
#[derive(Debug, Clone)]
pub struct SlowFrame<'data, 'headers, 'parser> {
    headers: &'headers Headers<'data>,
    raw: RawSlowFrame,
    filter: &'parser AppliedFilter,
}

impl super::seal::Sealed for SlowFrame<'_, '_, '_> {}

impl super::Frame for SlowFrame<'_, '_, '_> {
    type Value = SlowValue;

    #[inline]
    fn len(&self) -> usize {
        self.filter.len()
    }

    fn get_raw(&self, index: usize) -> Option<u32> {
        let index = self.filter.get(index)?;
        Some(self.raw.0[index])
    }

    fn get(&self, index: usize) -> Option<Self::Value> {
        let frame_def = self.headers.slow_frame_def();
        let index = self.filter.get(index)?;

        let def = &frame_def.fields[index];
        let raw = self.raw.0[index];

        let firmware = self.headers.internal_firmware;
        let value = match def.unit {
            SlowUnit::FlightMode => SlowValue::FlightMode(units::FlightModeSet::new(raw, firmware)),
            SlowUnit::State => SlowValue::State(units::StateSet::new(raw, firmware)),
            SlowUnit::FailsafePhase => {
                SlowValue::FailsafePhase(units::FailsafePhase::new(raw, firmware))
            }
            SlowUnit::Boolean => {
                if raw > 1 {
                    tracing::debug!("invalid boolean ({raw:0>#8x})");
                }

                SlowValue::Boolean(raw != 0)
            }
            SlowUnit::Unitless => SlowValue::new_unitless(raw, def.signed),
        };

        Some(value)
    }
}

impl<'data, 'headers, 'parser> SlowFrame<'data, 'headers, 'parser> {
    pub(crate) fn new(
        headers: &'headers Headers<'data>,
        raw: RawSlowFrame,
        filter: &'parser AppliedFilter,
    ) -> Self {
        Self {
            headers,
            raw,
            filter,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RawSlowFrame(Vec<u32>);

impl RawSlowFrame {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlowValue {
    FlightMode(units::FlightModeSet),
    State(units::StateSet),
    FailsafePhase(units::FailsafePhase),
    Boolean(bool),
    Unsigned(u32),
    Signed(i32),
}

impl SlowValue {
    const fn new_unitless(value: u32, signed: bool) -> Self {
        if signed {
            Self::Signed(as_i32(value))
        } else {
            Self::Unsigned(value)
        }
    }
}

impl From<SlowValue> for super::Value {
    fn from(value: SlowValue) -> Self {
        match value {
            SlowValue::FlightMode(m) => Self::FlightMode(m),
            SlowValue::State(s) => Self::State(s),
            SlowValue::FailsafePhase(p) => Self::FailsafePhase(p),
            SlowValue::Boolean(b) => Self::Boolean(b),
            SlowValue::Unsigned(x) => Self::Unsigned(x),
            SlowValue::Signed(x) => Self::Signed(x),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlowUnit {
    FlightMode,
    State,
    FailsafePhase,
    Boolean,
    Unitless,
}

impl From<SlowUnit> for Unit {
    fn from(unit: SlowUnit) -> Self {
        match unit {
            SlowUnit::FlightMode => Self::FlightMode,
            SlowUnit::State => Self::State,
            SlowUnit::FailsafePhase => Self::FailsafePhase,
            SlowUnit::Boolean => Self::Boolean,
            SlowUnit::Unitless => Self::Unitless,
        }
    }
}
