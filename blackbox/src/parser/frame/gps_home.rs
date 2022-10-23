use alloc::vec::Vec;

use tracing::instrument;

use super::{read_field_values, FrameKind, FrameProperty};
use crate::parser::{Encoding, Headers, ParseError, ParseResult, Predictor, Reader};

#[derive(Debug, Clone)]
pub struct GpsHomeFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpsHomeValue {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpsHomeUnit {}

#[derive(Debug, Clone)]
pub(crate) struct GpsHomeFrameDef<'data>(pub(crate) Vec<GpsHomeFieldDef<'data>>);

impl<'data> GpsHomeFrameDef<'data> {
    pub(crate) fn builder() -> GpsHomeFrameDefBuilder<'data> {
        GpsHomeFrameDefBuilder::default()
    }

    #[instrument(level = "trace", name = "GpsHomeFrameDef::parse", skip_all)]
    pub(crate) fn parse(&self, data: &mut Reader, _headers: &Headers) -> ParseResult<GpsHomeFrame> {
        let _ = read_field_values(data, &self.0, |f| f.encoding)?;

        Ok(GpsHomeFrame)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct GpsHomeFieldDef<'data> {
    pub(crate) name: &'data str,
    pub(crate) predictor: Predictor,
    pub(crate) encoding: Encoding,
    // pub(crate) unit: GpsHomeUnit,
    pub(crate) signed: bool,
}

#[derive(Debug, Default)]
pub(crate) struct GpsHomeFrameDefBuilder<'data> {
    names: Option<&'data str>,
    predictors: Option<&'data str>,
    encodings: Option<&'data str>,
    signs: Option<&'data str>,
}

impl<'data> GpsHomeFrameDefBuilder<'data> {
    pub(crate) fn update(&mut self, property: FrameProperty, value: &'data str) {
        let value = Some(value);

        match property {
            FrameProperty::Name => self.names = value,
            FrameProperty::Predictor => self.predictors = value,
            FrameProperty::Encoding => self.encodings = value,
            FrameProperty::Signed => self.signs = value,
        }
    }

    pub(crate) fn parse(self) -> ParseResult<Option<GpsHomeFrameDef<'data>>> {
        let kind = FrameKind::Gps;

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

        let fields = (names.by_ref().zip(signs.by_ref()))
            .zip(predictors.by_ref().zip(encodings.by_ref()))
            .map(|((name, signed), (predictor, encoding))| {
                Ok(GpsHomeFieldDef {
                    name,
                    predictor: predictor?,
                    encoding: encoding?,
                    // unit: unit_from_name(name),
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

        Ok(Some(GpsHomeFrameDef(fields)))
    }
}
