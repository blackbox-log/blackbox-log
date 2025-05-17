use alloc::borrow::ToOwned as _;
use alloc::vec::Vec;

use tracing::instrument;

use super::{GpsUnit, RawGpsFrame};
use crate::frame::{
    self, DataFrameKind, DataFrameProperty, FieldDef, FieldDefDetails, FrameDef, GpsHomeFrame,
};
use crate::headers::{ParseError, ParseResult};
use crate::parser::{decode, Encoding, InternalResult};
use crate::predictor::{Predictor, PredictorContext};
use crate::utils::to_base_field;
use crate::{Headers, Reader, Unit};

/// The parsed frame definition for GPS frames.
#[derive(Debug, Clone)]
pub struct GpsFrameDef<'data> {
    pub(super) fields: Vec<GpsFieldDef<'data>>,
}

impl frame::seal::Sealed for GpsFrameDef<'_> {}

impl<'data> FrameDef<'data> for GpsFrameDef<'data> {
    type Unit = GpsUnit;

    #[inline]
    fn len(&self) -> usize {
        self.fields.len()
    }

    fn get<'a>(&'a self, index: usize) -> Option<FieldDef<'data, Self::Unit>>
    where
        'data: 'a,
    {
        self.fields.get(index).map(
            |&GpsFieldDef {
                 name, unit, signed, ..
             }| FieldDef { name, unit, signed },
        )
    }
}

impl<'data> GpsFrameDef<'data> {
    pub(crate) fn builder() -> GpsFrameDefBuilder<'data> {
        GpsFrameDefBuilder::default()
    }

    pub(crate) fn validate(
        &self,
        check_predictor: impl Fn(DataFrameKind, &'data str, Predictor, usize) -> ParseResult<()>,
        check_unit: impl Fn(DataFrameKind, &'data str, Unit) -> ParseResult<()>,
    ) -> ParseResult<()> {
        for (
            i,
            GpsFieldDef {
                name,
                predictor,
                unit,
                ..
            },
        ) in self.fields.iter().enumerate()
        {
            check_predictor(DataFrameKind::Gps, name, *predictor, i)?;
            check_unit(DataFrameKind::Gps, name, Unit::from(*unit))?;
        }

        Ok(())
    }

    #[instrument(level = "trace", name = "GpsFrameDef::parse", skip_all)]
    pub(crate) fn parse(
        &self,
        data: &mut Reader,
        headers: &Headers,
        last_main_time: Option<u64>,
        last_home: Option<&GpsHomeFrame>,
    ) -> InternalResult<RawGpsFrame> {
        let time = {
            let time = last_main_time.unwrap_or(0);
            let offset = decode::variable(data)?.into();

            let time = time.saturating_add(offset);
            tracing::trace!(time, offset);
            time
        };

        let raw = frame::read_field_values(data, &self.fields, |f| f.encoding)?;

        let ctx = PredictorContext::with_home(headers, last_home.map(|home| home.0));
        let mut values = Vec::with_capacity(raw.len());

        for (i, field) in self.fields.iter().enumerate() {
            let raw = raw[i];
            let signed = field.encoding.is_signed();

            trace_field!(pre, field = field, enc = field.encoding, raw = raw);

            let value = field.predictor.apply(raw, signed, None, &ctx);

            trace_field!(
                post,
                field = field,
                pred = field.predictor,
                final = value
            );

            values.push(value);
        }

        Ok(RawGpsFrame { time, values })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct GpsFieldDef<'data> {
    pub(crate) name: &'data str,
    pub(crate) predictor: Predictor,
    pub(crate) encoding: Encoding,
    pub(crate) unit: GpsUnit,
    pub(crate) signed: bool,
}

impl<'data> FieldDefDetails<'data> for &GpsFieldDef<'data> {
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
pub(crate) struct GpsFrameDefBuilder<'data> {
    names: Option<&'data str>,
    predictors: Option<&'data str>,
    encodings: Option<&'data str>,
    signs: Option<&'data str>,
}

impl<'data> GpsFrameDefBuilder<'data> {
    pub(crate) fn update(&mut self, property: DataFrameProperty, value: &'data str) {
        let value = Some(value);

        match property {
            DataFrameProperty::Name => self.names = value,
            DataFrameProperty::Predictor => self.predictors = value,
            DataFrameProperty::Encoding => self.encodings = value,
            DataFrameProperty::Signed => self.signs = value,
        }
    }

    pub(crate) fn parse(self) -> ParseResult<Option<GpsFrameDef<'data>>> {
        let kind = DataFrameKind::Gps;

        if self.names.is_none()
            && self.predictors.is_none()
            && self.encodings.is_none()
            && self.signs.is_none()
        {
            return Ok(None);
        }

        let mut names = frame::parse_names(kind, self.names)?;
        let mut predictors = frame::parse_predictors(kind, self.predictors)?;
        let mut encodings = frame::parse_encodings(kind, self.encodings)?;
        let mut signs = frame::parse_signs(kind, self.signs)?;

        let mut fields = (names.by_ref().zip(signs.by_ref()))
            .zip(predictors.by_ref().zip(encodings.by_ref()))
            .map(|((name, signed), (predictor, encoding))| {
                Ok(GpsFieldDef {
                    name,
                    predictor: predictor?,
                    encoding: encoding?,
                    unit: unit_from_name(name),
                    signed,
                })
            });

        if !matches!(
            fields.next().transpose()?,
            Some(GpsFieldDef {
                name: "time",
                predictor: Predictor::LastMainFrameTime,
                encoding: Encoding::Variable,
                ..
            })
        ) {
            return Err(ParseError::MissingField {
                frame: DataFrameKind::Gps,
                field: "time".to_owned(),
            });
        }

        let mut fields = fields.collect::<Result<Vec<_>, _>>()?;
        for (i, j) in (1..fields.len()).map(|i| (i - 1, i)) {
            if fields[i].predictor == Predictor::HomeLat
                && fields[j].predictor == Predictor::HomeLat
            {
                fields[j].predictor = Predictor::HomeLon;
            }
        }

        if names.next().is_some()
            || predictors.next().is_some()
            || encodings.next().is_some()
            || signs.next().is_some()
        {
            tracing::error!("not all gps definition headers are of equal length");
            return Err(ParseError::MalformedFrameDef(DataFrameKind::Gps));
        }

        Ok(Some(GpsFrameDef { fields }))
    }
}

fn unit_from_name(name: &str) -> GpsUnit {
    match to_base_field(name) {
        "GPS_coord" => GpsUnit::Coordinate,
        "GPS_altitude" => GpsUnit::Altitude,
        "GPS_speed" => GpsUnit::Velocity,
        "GPS_ground_course" => GpsUnit::Heading,
        _ => GpsUnit::Unitless,
    }
}
