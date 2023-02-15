use alloc::vec::Vec;

use tracing::instrument;

use super::{
    read_field_values, DataFrameKind, DataFrameProperty, FieldDef, FieldDefDetails, FrameDef, Unit,
};
use crate::filter::AppliedFilter;
use crate::headers::ParseResult;
use crate::parser::{Encoding, InternalResult};
use crate::predictor::{Predictor, PredictorContext};
use crate::utils::as_i32;
use crate::{units, Headers, Reader};

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

        let firmware = self.headers.firmware();
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlowUnit {
    FlightMode,
    State,
    FailsafePhase,
    Boolean,
    Unitless,
}

/// The parsed frame definition for slow frames.
#[derive(Debug, Clone)]
pub struct SlowFrameDef<'data> {
    fields: Vec<SlowFieldDef<'data>>,
}

impl super::seal::Sealed for SlowFrameDef<'_> {}

impl<'data> FrameDef<'data> for SlowFrameDef<'data> {
    type Unit = SlowUnit;

    #[inline]
    fn len(&self) -> usize {
        self.fields.len()
    }

    fn get<'a>(&'a self, index: usize) -> Option<FieldDef<'data, Self::Unit>>
    where
        'data: 'a,
    {
        self.fields.get(index).map(
            |&SlowFieldDef {
                 name, unit, signed, ..
             }| FieldDef { name, unit, signed },
        )
    }
}

impl<'data> SlowFrameDef<'data> {
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
        } in &self.fields
        {
            check_predictor(name, *predictor)?;
            check_unit(name, Unit::from(*unit))?;
        }

        Ok(())
    }

    #[instrument(level = "trace", name = "SlowFrameDef::parse", skip_all)]
    pub(crate) fn parse(
        &self,
        data: &mut Reader,
        headers: &Headers,
    ) -> InternalResult<RawSlowFrame> {
        let values = super::parse_impl(
            PredictorContext::new(headers),
            &read_field_values(data, &self.fields, |f| f.encoding)?,
            self.fields.iter(),
            |_, _| {},
        );

        Ok(RawSlowFrame(values))
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

impl<'data> FieldDefDetails<'data> for &SlowFieldDef<'data> {
    fn name(&self) -> &'data str {
        self.name
    }

    fn predictor(&self) -> Predictor {
        self.predictor
    }

    fn encoding(&self) -> Encoding {
        self.encoding
    }

    fn signed(&self) -> bool {
        self.signed
    }
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

        Ok(SlowFrameDef { fields })
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
