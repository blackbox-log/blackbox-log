use alloc::vec::Vec;

use tracing::instrument;

use super::{RawSlowFrame, SlowUnit};
use crate::frame::{self, DataFrameKind, DataFrameProperty, FieldDef, FieldDefDetails, FrameDef};
use crate::headers::{ParseError, ParseResult};
use crate::parser::{Encoding, InternalResult};
use crate::predictor::{Predictor, PredictorContext};
use crate::{Headers, Reader, Unit};

/// The parsed frame definition for slow frames.
#[derive(Debug, Clone)]
pub struct SlowFrameDef<'data> {
    pub(super) fields: Vec<SlowFieldDef<'data>>,
}

impl frame::seal::Sealed for SlowFrameDef<'_> {}

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
        check_predictor: impl Fn(DataFrameKind, &'data str, Predictor, usize) -> ParseResult<()>,
        check_unit: impl Fn(DataFrameKind, &'data str, Unit) -> ParseResult<()>,
    ) -> ParseResult<()> {
        for (
            i,
            SlowFieldDef {
                name,
                predictor,
                unit,
                ..
            },
        ) in self.fields.iter().enumerate()
        {
            check_predictor(DataFrameKind::Slow, name, *predictor, i)?;
            check_unit(DataFrameKind::Slow, name, Unit::from(*unit))?;
        }

        Ok(())
    }

    #[instrument(level = "trace", name = "SlowFrameDef::parse", skip_all)]
    pub(crate) fn parse(
        &self,
        data: &mut Reader,
        headers: &Headers,
    ) -> InternalResult<RawSlowFrame> {
        let values = frame::parse_impl(
            PredictorContext::new(headers),
            &frame::read_field_values(data, &self.fields, |f| f.encoding)?,
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

        let mut names = frame::parse_names(kind, self.names)?;
        let mut predictors = frame::parse_predictors(kind, self.predictors)?;
        let mut encodings = frame::parse_encodings(kind, self.encodings)?;
        let mut signs = frame::parse_signs(kind, self.signs)?;

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
            tracing::error!("not all slow definition headers are of equal length");
            return Err(ParseError::MalformedFrameDef(DataFrameKind::Slow));
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
