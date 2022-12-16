use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::iter;

use tracing::instrument;

use super::{
    read_field_values, DataFrameKind, DataFrameProperty, FieldDef, GpsHomeFrame, MainFrame, Unit,
};
use crate::parser::{decode, to_base_field, Encoding, InternalResult};
use crate::predictor::{Predictor, PredictorContext};
use crate::units::prelude::*;
use crate::units::FromRaw;
use crate::utils::as_i32;
use crate::{Headers, HeadersParseError, HeadersParseResult, Reader};

#[derive(Debug, Clone)]
pub(crate) struct GpsFrame {
    pub(crate) time: GpsValue,
    pub(crate) values: Vec<GpsValue>,
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
    Missing,
}

impl GpsFrame {
    pub(crate) fn get(&self, index: usize) -> Option<GpsValue> {
        match index {
            0 => Some(self.time),
            _ => self.values.get(index - 1).copied(),
        }
    }
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

#[derive(Debug, Clone)]
pub(crate) struct GpsFrameDef<'data>(pub(crate) Vec<GpsFieldDef<'data>>);

impl<'data> GpsFrameDef<'data> {
    pub(crate) fn len(&self) -> usize {
        1 + self.0.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<(&str, GpsUnit)> {
        if index == 0 {
            Some(("time", GpsUnit::FrameTime))
        } else {
            self.0.get(index - 1).map(|field| (field.name, field.unit))
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, GpsUnit)> {
        iter::once(("time", GpsUnit::FrameTime)).chain(self.0.iter().map(|f| (f.name, f.unit)))
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
        } in &self.0
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
        last_main: Option<&MainFrame>,
        last_home: Option<&GpsHomeFrame>,
    ) -> InternalResult<GpsFrame> {
        let time = {
            let time = last_main.map_or(0, |main| main.time);
            let offset = decode::variable(data)?.into();

            let time = time.saturating_add(offset);
            tracing::trace!(time, offset);
            GpsValue::FrameTime(Time::from_raw(time, headers))
        };

        let raw = read_field_values(data, &self.0, |f| f.encoding)?;

        let ctx = PredictorContext::with_home(headers, last_home.map(|home| home.0));
        let mut values = Vec::with_capacity(raw.len());

        for (i, field) in self.0.iter().enumerate() {
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

            values.push(match field.unit {
                GpsUnit::FrameTime => unreachable!(),
                GpsUnit::Coordinate => {
                    assert!(field.signed);
                    let value = as_i32(value);

                    GpsValue::Coordinate(f64::from(value) / 10000000.)
                }
                GpsUnit::Altitude => {
                    let altitude = if field.signed {
                        as_i32(value).into()
                    } else {
                        value.into()
                    };

                    GpsValue::Altitude(Length::new::<meter>(altitude))
                }
                GpsUnit::Velocity => {
                    assert!(!field.signed);
                    GpsValue::Velocity(Velocity::from_raw(value, headers))
                }
                GpsUnit::Heading => {
                    assert!(!field.signed);
                    GpsValue::Heading(f64::from(value) / 10.)
                }
                GpsUnit::Unitless => GpsValue::new_unitless(value, signed),
            });
        }

        Ok(GpsFrame { time, values })
    }

    pub(crate) fn empty_frame(&self) -> GpsFrame {
        let values = iter::repeat(GpsValue::Missing).take(self.0.len()).collect();
        GpsFrame {
            time: GpsValue::Missing,
            values,
        }
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

        match fields.next().transpose()? {
            Some(GpsFieldDef {
                name: "time",
                predictor: Predictor::LastMainFrameTime,
                encoding: Encoding::Variable,
                ..
            }) => {}
            def => {
                tracing::debug!(?def, "found invalid gps time field definition");
                return Err(HeadersParseError::MissingField {
                    frame: DataFrameKind::Gps,
                    field: "time".to_owned(),
                });
            }
        }

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

        Ok(Some(GpsFrameDef(fields)))
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
