mod def;

use alloc::vec::Vec;

pub use self::def::*;
use super::{DataFrameKind, FrameKind, Unit};
use crate::data::MainFrameHistory;
use crate::filter::AppliedFilter;
use crate::parser::InternalResult;
use crate::units::prelude::*;
use crate::{units, Headers, Reader};

/// Data parsed from a main frame.
#[derive(Debug)]
pub struct MainFrame<'data, 'headers, 'parser> {
    headers: &'headers Headers<'data>,
    raw: &'parser RawMainFrame,
    filter: &'parser AppliedFilter,
}

impl super::seal::Sealed for MainFrame<'_, '_, '_> {}

impl super::Frame for MainFrame<'_, '_, '_> {
    type Value = MainValue;

    #[inline]
    fn len(&self) -> usize {
        self.filter.len()
    }

    fn get_raw(&self, index: usize) -> Option<u32> {
        let index = self.filter.get(index)?;

        let value = if index == 0 {
            self.raw.iteration
        } else {
            self.raw.values[index - 1]
        };

        Some(value)
    }

    fn get(&self, index: usize) -> Option<MainValue> {
        let frame_def = self.headers.main_frame_def();
        let index = self.filter.get(index)?;

        if index == 0 {
            return Some(MainValue::Unsigned(self.raw.iteration));
        }
        let index = index - 1;

        let def = &frame_def.fields[index];
        let raw = self.raw.values[index];

        let value = match def.unit {
            MainUnit::Amperage => {
                debug_assert!(def.signed);
                let raw = raw.cast_signed();
                MainValue::Amperage(units::new::current(raw))
            }
            MainUnit::Voltage => {
                debug_assert!(!def.signed);
                MainValue::Voltage(units::new::vbat(raw))
            }
            MainUnit::Acceleration => {
                debug_assert!(def.signed);
                let raw = raw.cast_signed();
                MainValue::Acceleration(units::new::acceleration(raw, self.headers))
            }
            MainUnit::Rotation => {
                debug_assert!(def.signed);
                let raw = raw.cast_signed();
                MainValue::Rotation(units::new::angular_velocity(raw, self.headers))
            }
            MainUnit::Unitless => MainValue::new_unitless(raw, def.signed),
        };

        Some(value)
    }
}

impl<'data, 'headers, 'parser> MainFrame<'data, 'headers, 'parser> {
    pub(crate) fn new(
        headers: &'headers Headers<'data>,
        raw: &'parser RawMainFrame,
        filter: &'parser AppliedFilter,
    ) -> Self {
        Self {
            headers,
            raw,
            filter,
        }
    }

    /// Returns the parsed time since power on.
    #[inline]
    pub fn time(&self) -> Time {
        units::new::time(self.raw.time)
    }

    /// Returns the raw microsecond counter since power on.
    ///
    /// **Note:** This does not currently handle overflow of the transmitted
    /// 32bit counter. See [#54](https://github.com/blackbox-log/blackbox-log/issues/54).
    #[inline]
    pub fn time_raw(&self) -> u64 {
        self.raw.time
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RawMainFrame {
    intra: bool,
    pub(crate) iteration: u32,
    pub(crate) time: u64,
    pub(crate) values: Vec<u32>,
}

impl RawMainFrame {
    pub(crate) fn parse(
        data: &mut Reader,
        headers: &Headers,
        kind: FrameKind,
        history: &MainFrameHistory,
    ) -> InternalResult<Self> {
        let last = history.last();
        let def = headers.main_frame_def();

        if kind == FrameKind::Data(DataFrameKind::Intra) {
            def.parse_intra(data, headers, last)
        } else {
            let skipped = 0; // FIXME

            def.parse_inter(data, headers, last, history.last_last(), skipped)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainValue {
    Amperage(ElectricCurrent),
    Voltage(ElectricPotential),
    Acceleration(Acceleration),
    Rotation(AngularVelocity),
    Unsigned(u32),
    Signed(i32),
}

impl MainValue {
    const fn new_unitless(value: u32, signed: bool) -> Self {
        if signed {
            Self::Signed(value.cast_signed())
        } else {
            Self::Unsigned(value)
        }
    }
}

impl From<MainValue> for super::Value {
    fn from(value: MainValue) -> Self {
        match value {
            MainValue::Amperage(a) => Self::Amperage(a),
            MainValue::Voltage(v) => Self::Voltage(v),
            MainValue::Acceleration(a) => Self::Acceleration(a),
            MainValue::Rotation(r) => Self::Rotation(r),
            MainValue::Unsigned(x) => Self::Unsigned(x),
            MainValue::Signed(x) => Self::Signed(x),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MainUnit {
    Amperage,
    Voltage,
    Acceleration,
    Rotation,
    Unitless,
}

impl From<MainUnit> for Unit {
    fn from(unit: MainUnit) -> Self {
        match unit {
            MainUnit::Amperage => Self::Amperage,
            MainUnit::Voltage => Self::Voltage,
            MainUnit::Acceleration => Self::Acceleration,
            MainUnit::Rotation => Self::Rotation,
            MainUnit::Unitless => Self::Unitless,
        }
    }
}
