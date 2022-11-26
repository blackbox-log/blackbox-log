use alloc::vec::Vec;
use core::iter;

use tracing::instrument;

use super::{read_field_values, DataFrameKind, DataFrameProperty, Unit};
use crate::parser::{Encoding, InternalResult, ParseResult};
use crate::predictor::{Predictor, PredictorContext};
use crate::utils::as_signed;
use crate::{units, Headers, Reader};

#[derive(Debug, Clone)]
pub(crate) struct SlowFrame {
    pub(crate) values: Vec<SlowValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SlowValue {
    FlightMode(units::FlightModeSet),
    State(units::StateSet),
    FailsafePhase(units::FailsafePhase),
    Boolean(bool),
    Unsigned(u32),
    Signed(i32),
    Missing,
}

impl SlowValue {
    const fn new_unitless(value: u32, signed: bool) -> Self {
        if signed {
            Self::Signed(as_signed(value))
        } else {
            Self::Unsigned(value)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SlowUnit {
    FlightMode,
    State,
    FailsafePhase,
    Boolean,
    Unitless,
}

#[derive(Debug, Clone)]
#[cfg_attr(fuzzing, derive(Default))]
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

    pub(crate) fn validate(
        &self,
        check_predictor: impl Fn(&'data str, Predictor) -> ParseResult<()>,
        check_unit: impl Fn(&'data str, Unit) -> ParseResult<()>,
    ) -> ParseResult<()> {
        for SlowFieldDef {
            name,
            predictor,
            unit,
            ..
        } in &self.0
        {
            check_predictor(name, *predictor)?;
            check_unit(name, Unit::from(*unit))?;
        }

        Ok(())
    }

    #[instrument(level = "trace", name = "SlowFrameDef::parse", skip_all)]
    pub(crate) fn parse(&self, data: &mut Reader, headers: &Headers) -> InternalResult<SlowFrame> {
        let raw = read_field_values(data, &self.0, |f| f.encoding)?;

        let ctx = PredictorContext::new(headers);
        let values = raw
            .iter()
            .zip(self.0.iter())
            .map(|(&raw_value, field)| {
                let value = field.predictor.apply(raw_value, field.signed, None, &ctx);

                tracing::trace!(
                    field = field.name,
                    encoding = ?field.encoding,
                    predictor = ?field.predictor,
                    raw = raw_value,
                    value,
                );

                let firmware = headers.firmware_kind;
                match field.unit {
                    SlowUnit::FlightMode => {
                        SlowValue::FlightMode(units::FlightModeSet::new(value, firmware))
                    }
                    SlowUnit::State => SlowValue::State(units::StateSet::new(value, firmware)),
                    SlowUnit::FailsafePhase => {
                        SlowValue::FailsafePhase(units::FailsafePhase::new(value, firmware))
                    }
                    SlowUnit::Boolean => {
                        if value > 1 {
                            tracing::debug!("invalid boolean ({value:0>#8x})");
                        }

                        SlowValue::Boolean(value != 0)
                    }
                    SlowUnit::Unitless => SlowValue::new_unitless(value, field.signed),
                }
            })
            .collect::<Vec<_>>();

        Ok(SlowFrame { values })
    }

    pub(crate) fn empty_frame(&self) -> SlowFrame {
        let values = iter::repeat(SlowValue::Missing).take(self.len()).collect();
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
            .collect::<Result<Vec<_>, _>>()?;

        if names.next().is_some()
            || predictors.next().is_some()
            || encodings.next().is_some()
            || signs.next().is_some()
        {
            tracing::warn!("not all slow frame definition headers are of equal length");
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
