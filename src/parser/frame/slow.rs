use super::{count_fields_with_same_encoding, Frame, FrameKind, FrameProperty};
use crate::parser::{Config, Encoding, Headers, ParseError, ParseResult, Predictor, Reader};
use alloc::vec::Vec;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct SlowFrame {
    values: Vec<i64>,
}

impl Frame for SlowFrame {
    fn values(&self) -> &[i64] {
        &self.values
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SlowFrameDef<'data>(pub(crate) Vec<SlowFieldDef<'data>>);

impl<'data> SlowFrameDef<'data> {
    pub(crate) fn builder() -> SlowFrameDefBuilder<'data> {
        SlowFrameDefBuilder::default()
    }

    #[instrument(level = "trace", name = "SlowFrameDef::parse", skip_all)]
    pub(crate) fn parse(
        &self,
        data: &mut Reader,
        config: &Config,
        headers: &Headers,
    ) -> ParseResult<SlowFrame> {
        let mut fields = self.0.iter().peekable();
        let mut values = Vec::with_capacity(self.0.len());

        while let Some(field) = fields.next() {
            let encoding = field.encoding;
            let extra = encoding.max_chunk_size() - 1;
            let extra = count_fields_with_same_encoding(&mut fields, extra, |&field| {
                field.encoding == encoding
            });

            values.append(&mut encoding.decode(data, headers.version, extra)?);
        }

        for i in 0..values.len() {
            let field = &self.0[i];
            let raw = values[i];

            if !config.raw {
                values[i] = field
                    .predictor
                    .apply(headers, raw, &values, None, None, 0)?;
            }

            tracing::trace!(
                field = field.name,
                encoding = ?field.encoding,
                predictor = ?field.predictor,
                raw,
                value = values[i],
            );

            // TODO: check field.signed
        }

        Ok(SlowFrame { values })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SlowFieldDef<'data> {
    pub(crate) name: &'data str,
    pub(crate) predictor: Predictor,
    pub(crate) encoding: Encoding,
}

#[derive(Debug, Default)]
pub(crate) struct SlowFrameDefBuilder<'data> {
    pub(crate) names: Option<&'data str>,
    pub(crate) predictors: Option<&'data str>,
    pub(crate) encodings: Option<&'data str>,
}

impl<'data> SlowFrameDefBuilder<'data> {
    pub(crate) fn update(&mut self, property: FrameProperty, value: &'data str) {
        let value = Some(value);

        match property {
            FrameProperty::Name => self.names = value,
            FrameProperty::Predictor => self.predictors = value,
            FrameProperty::Encoding => self.encodings = value,
        }
    }

    pub(crate) fn parse(self) -> ParseResult<SlowFrameDef<'data>> {
        let kind = FrameKind::Slow;

        let mut names = super::parse_names(kind, self.names)?;
        let mut predictors = super::parse_predictors(kind, self.predictors)?;
        let mut encodings = super::parse_encodings(kind, self.encodings)?;

        let fields = names
            .by_ref()
            .zip(predictors.by_ref())
            .zip(encodings.by_ref())
            .map(|((name, predictor), encoding)| {
                Ok(SlowFieldDef {
                    name,
                    predictor: predictor?,
                    encoding: encoding?,
                })
            })
            .collect::<ParseResult<Vec<_>>>()?;

        if names.next().is_none() || predictors.next().is_none() || encodings.next().is_none() {
            tracing::error!("all `Field *` headers must have the same number of elements");
            return Err(ParseError::Corrupted);
        }

        Ok(SlowFrameDef(fields))
    }
}
