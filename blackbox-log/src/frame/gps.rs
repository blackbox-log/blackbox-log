use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::iter;

use tracing::instrument;

use super::{read_field_values, DataFrameKind, DataFrameProperty, FieldDef, GpsHomeFrame, Unit};
use crate::filter::{AppliedFilter, FieldFilter};
use crate::parser::{decode, to_base_field, Encoding, InternalResult};
use crate::predictor::{Predictor, PredictorContext};
use crate::units::prelude::*;
use crate::units::FromRaw;
use crate::utils::as_i32;
use crate::{Headers, HeadersParseError, HeadersParseResult, Reader};

/// Data parsed from a GPS frame.
#[derive(Debug, Clone)]
pub struct GpsFrame<'data, 'headers> {
    headers: &'headers Headers<'data>,
    raw: RawGpsFrame,
}

impl super::seal::Seal for GpsFrame<'_, '_> {}

impl super::Frame for GpsFrame<'_, '_> {
    type Value = GpsValue;

    fn get(&self, index: usize) -> Option<Self::Value> {
        let value = if index == 0 {
            GpsValue::FrameTime(Time::from_raw(self.raw.time, self.headers))
        } else {
            let def = self.headers.gps_frames.as_ref().unwrap();
            let index = def.filter.get(index - 1)?;
            let def = &def.fields[index];
            let raw = self.raw.values[index];

            match def.unit {
                GpsUnit::FrameTime => unreachable!(),
                GpsUnit::Coordinate => {
                    assert!(def.signed);
                    let value = as_i32(raw);

                    GpsValue::Coordinate(f64::from(value) / 10000000.)
                }
                GpsUnit::Altitude => {
                    let altitude = if def.signed {
                        as_i32(raw).into()
                    } else {
                        raw.into()
                    };

                    GpsValue::Altitude(Length::new::<meter>(altitude))
                }
                GpsUnit::Velocity => {
                    assert!(!def.signed);
                    GpsValue::Velocity(Velocity::from_raw(raw, self.headers))
                }
                GpsUnit::Heading => {
                    assert!(!def.signed);
                    GpsValue::Heading(f64::from(raw) / 10.)
                }
                GpsUnit::Unitless => GpsValue::new_unitless(raw, def.signed),
            }
        };

        Some(value)
    }
}

impl<'data, 'headers> GpsFrame<'data, 'headers> {
    pub(crate) fn new(headers: &'headers Headers<'data>, raw: RawGpsFrame) -> Self {
        Self { headers, raw }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RawGpsFrame {
    pub(crate) time: u64,
    pub(crate) values: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpsValue {
    FrameTime(Time),
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
            Self::Signed(as_i32(value))
        } else {
            Self::Unsigned(value)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpsUnit {
    FrameTime,
    Coordinate,
    Altitude,
    Velocity,
    Heading,
    Unitless,
}

/// The parsed frame definition for GPS frames.
#[derive(Debug, Clone)]
pub struct GpsFrameDef<'data> {
    pub(crate) time: GpsFieldDef<'data>,
    pub(crate) fields: Vec<GpsFieldDef<'data>>,
    filter: AppliedFilter,
}

impl super::seal::Seal for GpsFrameDef<'_> {}

impl<'data> super::FrameDef<'data> for GpsFrameDef<'data> {
    type Unit = GpsUnit;

    fn len(&self) -> usize {
        1 + self.filter.len()
    }

    fn get<'a>(&'a self, index: usize) -> Option<(&'data str, GpsUnit)>
    where
        'data: 'a,
    {
        let field = match index {
            0 => Some(&self.time),
            _ => self.fields.get(self.filter.get(index - 1)?),
        };

        field.map(|f| (f.name, f.unit))
    }

    fn clear_filter(&mut self) {
        self.filter = AppliedFilter::new_unfiltered(self.fields.len());
    }

    fn apply_filter(&mut self, filter: &FieldFilter) {
        self.filter = filter.apply(self.fields.iter().map(|f| f.name));
    }
}

impl<'data> GpsFrameDef<'data> {
    /// Iterates over the name and unit of each field.
    pub fn iter(&self) -> impl Iterator<Item = (&str, GpsUnit)> {
        iter::once(&self.time)
            .chain(self.filter.iter().map(|i| &self.fields[i]))
            .map(|f| (f.name, f.unit))
    }

    /// Iterates over the names of each field.
    pub fn iter_names(&self) -> impl Iterator<Item = &str> {
        iter::once(&self.time)
            .chain(self.filter.iter().map(|i| &self.fields[i]))
            .map(|f| f.name)
    }

    pub(crate) fn builder() -> GpsFrameDefBuilder<'data> {
        GpsFrameDefBuilder::default()
    }

    pub(crate) fn validate(
        &self,
        check_predictor: impl Fn(&'data str, Predictor) -> HeadersParseResult<()>,
        check_unit: impl Fn(&'data str, Unit) -> HeadersParseResult<()>,
    ) -> HeadersParseResult<()> {
        for GpsFieldDef {
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

        let raw = read_field_values(data, &self.fields, |f| f.encoding)?;

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

impl<'data> FieldDef<'data> for &GpsFieldDef<'data> {
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

    pub(crate) fn parse(self) -> HeadersParseResult<Option<GpsFrameDef<'data>>> {
        let kind = DataFrameKind::Gps;

        if self.names.is_none()
            && self.predictors.is_none()
            && self.encodings.is_none()
            && self.signs.is_none()
        {
            return Ok(None);
        }

        let mut names = super::parse_names(kind, self.names)?;
        let mut predictors = super::parse_predictors(kind, self.predictors)?;
        let mut encodings = super::parse_encodings(kind, self.encodings)?;
        let mut signs = super::parse_signs(kind, self.signs)?;

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

        let Some(time @ GpsFieldDef {
            name: "time",
            predictor: Predictor::LastMainFrameTime,
            encoding: Encoding::Variable,
            ..
        }) = fields.next().transpose()? else {
            return Err(HeadersParseError::MissingField {
                frame: DataFrameKind::Gps,
                field: "time".to_owned()
            });
        };

        let mut fields = fields.collect::<Result<Vec<_>, _>>()?;
        for (i, j) in (1..fields.len()).into_iter().map(|i| (i - 1, i)) {
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
            tracing::warn!("not all GPS definition headers are of equal length");
        }

        let filter = AppliedFilter::new_unfiltered(fields.len());

        Ok(Some(GpsFrameDef {
            time,
            fields,
            filter,
        }))
    }
}

fn unit_from_name(name: &str) -> GpsUnit {
    match to_base_field(name) {
        "time" => GpsUnit::FrameTime,
        "GPS_coord" => GpsUnit::Coordinate,
        "GPS_altitude" => GpsUnit::Altitude,
        "GPS_speed" => GpsUnit::Velocity,
        "GPS_ground_course" => GpsUnit::Heading,
        _ => GpsUnit::Unitless,
    }
}
