use alloc::vec::Vec;
use core::iter;

use tracing::instrument;

use super::{read_field_values, DataFrameKind, DataFrameProperty};
use crate::parser::{as_signed, Encoding, Headers, ParseError, ParseResult, Predictor, Reader};
use crate::units;

#[derive(Debug, Clone)]
pub struct SlowFrame {
    pub(crate) values: Vec<SlowValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlowValue {
    FlightMode(units::FlightModeSet),
    State(units::StateSet),
    FailsafePhase(units::FailsafePhaseSet),
    Boolean(bool),
    Unsigned(u32),
    Signed(i32),
}

impl SlowValue {
    fn new_unitless(value: u32, signed: bool) -> Self {
        if signed {
            Self::Signed(as_signed(value))
        } else {
            Self::Unsigned(value)
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

#[derive(Debug, Clone)]
pub(crate) struct SlowFrameDef<'data>(pub(crate) Vec<SlowFieldDef<'data>>);

impl<'data> SlowFrameDef<'data> {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<(&str, SlowUnit)> {
        self.0.get(index).map(|f| (f.name, f.unit))
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, SlowUnit)> {
        self.0.iter().map(|f| (f.name, f.unit))
    }

    pub(crate) fn builder() -> SlowFrameDefBuilder<'data> {
        SlowFrameDefBuilder::default()
    }

    #[instrument(level = "trace", name = "SlowFrameDef::parse", skip_all)]
    pub(crate) fn parse(&self, data: &mut Reader, headers: &Headers) -> ParseResult<SlowFrame> {
        let raw = read_field_values(data, &self.0, |f| f.encoding)?;

        let values = raw
            .iter()
            .zip(self.0.iter())
            .map(|(&raw_value, field)| {
                let value =
                    field
                        .predictor
                        .apply(headers, raw_value, field.signed, &raw, None, None, 0)?;

                tracing::trace!(
                    field = field.name,
                    encoding = ?field.encoding,
                    predictor = ?field.predictor,
                    raw = raw_value,
                    value,
                );

                let firmware = headers.firmware_kind;
                let value = match field.unit {
                    SlowUnit::FlightMode => {
                        SlowValue::FlightMode(units::FlightModeSet::new(value, firmware))
                    }
                    SlowUnit::State => SlowValue::State(units::StateSet::new(value, firmware)),
                    SlowUnit::FailsafePhase => {
                        SlowValue::FailsafePhase(units::FailsafePhaseSet::new(value, firmware))
                    }
                    SlowUnit::Boolean => SlowValue::Boolean(match value {
                        0 => false,
                        1 => true,
                        _ => {
                            tracing::debug!("invalid boolean ({value})");
                            return Err(ParseError::Corrupted);
                        }
                    }),
                    SlowUnit::Unitless => SlowValue::new_unitless(value, field.signed),
                };

                Ok(value)
            })
            .collect::<ParseResult<Vec<_>>>()?;

        Ok(SlowFrame { values })
    }

    pub(crate) fn default_frame(&self, headers: &Headers) -> SlowFrame {
        let mut i = 0;
        let values = iter::from_fn(|| {
            self.0.get(i).map(|def| {
                i += 1;

                let firmware = headers.firmware_kind;
                match def.unit {
                    SlowUnit::FlightMode => {
                        SlowValue::FlightMode(units::FlightModeSet::new(0, firmware))
                    }
                    SlowUnit::State => SlowValue::State(units::StateSet::new(0, firmware)),
                    SlowUnit::FailsafePhase => {
                        SlowValue::FailsafePhase(units::FailsafePhaseSet::new(0, firmware))
                    }
                    SlowUnit::Boolean => SlowValue::Boolean(false),
                    SlowUnit::Unitless => SlowValue::new_unitless(0, def.signed),
                }
            })
        })
        .collect();

        SlowFrame { values }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SlowFieldDef<'data> {
    pub(crate) name: &'data str,
    pub(crate) predictor: Predictor,
    pub(crate) encoding: Encoding,
    pub(crate) unit: SlowUnit,
    pub(crate) signed: bool,
}

#[derive(Debug, Default)]
pub(crate) struct SlowFrameDefBuilder<'data> {
    names: Option<&'data str>,
    predictors: Option<&'data str>,
    encodings: Option<&'data str>,
    signs: Option<&'data str>,
}

impl<'data> SlowFrameDefBuilder<'data> {
    pub(crate) fn update(&mut self, property: DataFrameProperty, value: &'data str) {
        let value = Some(value);

        match property {
            DataFrameProperty::Name => self.names = value,
            DataFrameProperty::Predictor => self.predictors = value,
            DataFrameProperty::Encoding => self.encodings = value,
            DataFrameProperty::Signed => self.signs = value,
        }
    }

    pub(crate) fn parse(self) -> ParseResult<SlowFrameDef<'data>> {
        let kind = DataFrameKind::Slow;

        let mut names = super::parse_names(kind, self.names)?;
        let mut predictors = super::parse_predictors(kind, self.predictors)?;
        let mut encodings = super::parse_encodings(kind, self.encodings)?;
        let mut signs = super::parse_signs(kind, self.signs)?;

        let fields = (names.by_ref().zip(signs.by_ref()))
            .zip(predictors.by_ref().zip(encodings.by_ref()))
            .map(|((name, signed), (predictor, encoding))| {
                Ok(SlowFieldDef {
                    name,
                    predictor: predictor?,
                    encoding: encoding?,
                    unit: unit_from_name(name),
                    signed,
                })
            })
            .collect::<ParseResult<Vec<_>>>()?;

        if names.next().is_some()
            || predictors.next().is_some()
            || encodings.next().is_some()
            || signs.next().is_some()
        {
            tracing::error!("all `Field *` headers must have the same number of elements");
            return Err(ParseError::Corrupted);
        }

        Ok(SlowFrameDef(fields))
    }
}

fn unit_from_name(name: &str) -> SlowUnit {
    match name {
        "flightModeFlags" => SlowUnit::FlightMode,
        "stateFlags" => SlowUnit::State,
        "failsafePhase" => SlowUnit::FailsafePhase,
        "rxSignalReceived" | "rxFlightChannelsValid" => SlowUnit::Boolean,
        _ => SlowUnit::Unitless,
    }
}
