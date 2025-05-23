use alloc::borrow::ToOwned as _;
use alloc::vec::Vec;

use tracing::instrument;

use super::{read_field_values, DataFrameKind, DataFrameProperty};
use crate::headers::{ParseError, ParseResult};
use crate::parser::{Encoding, InternalResult};
use crate::predictor::{Predictor, PredictorContext};
use crate::{Headers, Reader};

#[derive(Debug, Clone)]
pub(crate) struct GpsHomeFrame(pub(crate) GpsPosition);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GpsPosition {
    pub(crate) latitude: i32,
    pub(crate) longitude: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct GpsHomeFrameDef<'data>([GpsHomeFieldDef<'data>; 2], Vec<Encoding>);

impl<'data> GpsHomeFrameDef<'data> {
    pub(crate) fn builder() -> GpsHomeFrameDefBuilder<'data> {
        GpsHomeFrameDefBuilder::default()
    }

    pub(crate) fn validate(
        &self,
        check_predictor: impl Fn(DataFrameKind, &'data str, Predictor, usize) -> ParseResult<()>,
        _check_unit: impl Fn(DataFrameKind, &'data str, super::Unit) -> ParseResult<()>,
    ) -> ParseResult<()> {
        for (
            i,
            GpsHomeFieldDef {
                name, predictor, ..
            },
        ) in self.0.iter().enumerate()
        {
            check_predictor(DataFrameKind::GpsHome, name, *predictor, i)?;
        }

        Ok(())
    }

    #[instrument(level = "trace", name = "GpsHomeFrameDef::parse", skip_all)]
    pub(crate) fn parse(
        &self,
        data: &mut Reader,
        headers: &Headers,
    ) -> InternalResult<GpsHomeFrame> {
        let raw = read_field_values(data, &self.0, |f| f.encoding)?;
        let _ = read_field_values(data, &self.1, |&f| f)?;

        let ctx = PredictorContext::new(headers);
        let values = raw
            .iter()
            .zip(self.0.iter())
            .map(|(&raw_value, field)| {
                let value = field.predictor.apply(raw_value, true, None, &ctx);

                tracing::trace!(
                    field = field.name,
                    encoding = ?field.encoding,
                    predictor = ?field.predictor,
                    raw = raw_value,
                    value,
                );

                value.cast_signed()
            })
            .collect::<Vec<_>>();

        // `values` can only have two elements thanks to zipping with `self.0`
        let [latitude, longitude, ..] = values[..] else {
            unreachable!()
        };

        Ok(GpsHomeFrame(GpsPosition {
            latitude,
            longitude,
        }))
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct GpsHomeFieldDef<'data> {
    pub(crate) name: &'data str,
    pub(crate) predictor: Predictor,
    pub(crate) encoding: Encoding,
}

#[derive(Debug, Default)]
pub(crate) struct GpsHomeFrameDefBuilder<'data> {
    names: Option<&'data str>,
    predictors: Option<&'data str>,
    encodings: Option<&'data str>,
    signs: Option<&'data str>,
}

impl<'data> GpsHomeFrameDefBuilder<'data> {
    pub(crate) fn update(&mut self, property: DataFrameProperty, value: &'data str) {
        let value = Some(value);

        match property {
            DataFrameProperty::Name => self.names = value,
            DataFrameProperty::Predictor => self.predictors = value,
            DataFrameProperty::Encoding => self.encodings = value,
            DataFrameProperty::Signed => self.signs = value,
        }
    }

    pub(crate) fn parse(self) -> ParseResult<Option<GpsHomeFrameDef<'data>>> {
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

        let mut fields =
            (names.by_ref().zip(signs.by_ref())).zip(predictors.by_ref().zip(encodings.by_ref()));

        let latitude =
            if let Some(((name @ "GPS_home[0]", true), (predictor, encoding))) = fields.next() {
                GpsHomeFieldDef {
                    name,
                    predictor: predictor?,
                    encoding: encoding?,
                }
            } else {
                tracing::error!("missing GPS_home[0] field definition");
                return Err(ParseError::MissingField {
                    frame: DataFrameKind::GpsHome,
                    field: "GPS_home[0]".to_owned(),
                });
            };

        let longitude =
            if let Some(((name @ "GPS_home[1]", true), (predictor, encoding))) = fields.next() {
                GpsHomeFieldDef {
                    name,
                    predictor: predictor?,
                    encoding: encoding?,
                }
            } else {
                tracing::error!("missing GPS_home[1] field definition");
                return Err(ParseError::MissingField {
                    frame: DataFrameKind::GpsHome,
                    field: "GPS_home[1]".to_owned(),
                });
            };

        let rest = fields
            .map(|(_, (_, encoding))| encoding)
            .collect::<Result<Vec<_>, _>>()?;

        if !rest.is_empty() {
            tracing::warn!(
                "expected only GPS_home[0] & GPS_home[1] fields in gps home frames, found {} more",
                rest.len()
            );
        }

        if names.next().is_some()
            || predictors.next().is_some()
            || encodings.next().is_some()
            || signs.next().is_some()
        {
            tracing::error!("not all GPS home definition headers are of equal length");
            return Err(ParseError::MalformedFrameDef(DataFrameKind::GpsHome));
        }

        Ok(Some(GpsHomeFrameDef([latitude, longitude], rest)))
    }
}
