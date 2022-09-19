use super::{count_fields_with_same_encoding, Frame, FrameKind, FrameProperty};
use crate::parser::{
    decode, Config, Encoding, Headers, ParseError, ParseResult, Predictor, Reader,
};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct MainFrame {
    iteration: u32,
    time: i64,
    values: Vec<i64>,
}

impl MainFrame {
    pub fn iteration(&self) -> u32 {
        self.iteration
    }

    pub fn time(&self) -> i64 {
        self.time
    }
}

impl Frame for MainFrame {
    fn values(&self) -> &[i64] {
        &self.values
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MainFrameDef<'data> {
    pub(crate) iteration: MainFieldDef<'data>,
    pub(crate) time: MainFieldDef<'data>,
    pub(crate) fields: Vec<MainFieldDef<'data>>,
}

impl<'data> MainFrameDef<'data> {
    pub(crate) fn builder() -> MainFrameDefBuilder<'data> {
        MainFrameDefBuilder::default()
    }

    #[instrument(level = "trace", name = "MainFrameDef::parse_intra", skip_all)]
    pub(crate) fn parse_intra(
        &self,
        data: &mut Reader,
        config: &Config,
        headers: &Headers,
        last: Option<&MainFrame>,
    ) -> ParseResult<MainFrame> {
        let iteration = decode::variable(data)?;
        tracing::trace!(iteration);
        let time = decode::variable(data)?.into();
        tracing::trace!(time);

        let mut fields = self.fields.iter().peekable();
        let mut values = Vec::with_capacity(self.fields.len());

        while let Some(field) = fields.next() {
            let encoding = field.encoding_intra;
            let extra = encoding.max_chunk_size() - 1;
            let extra = count_fields_with_same_encoding(&mut fields, extra, |&field| {
                field.encoding_intra == encoding
            });

            let mut new = encoding.decode(data, headers.version, extra)?;
            debug_assert_eq!(extra + 1, new.len());
            values.append(&mut new);
        }

        debug_assert_eq!(values.len(), self.fields.len());
        for (i, value) in values.iter_mut().enumerate() {
            let field = &self.fields[i];
            let raw = *value;

            let last = last.map(|l| l.values[i]);

            if !config.raw {
                *value = field.predictor_intra.apply(headers, *value, last, None, 0);
            }

            tracing::trace!(
                field = field.name,
                encoding = ?field.encoding_intra,
                predictor = ?field.predictor_intra,
                raw,
                value,
            );

            // TODO: check field.signed
        }

        Ok(MainFrame {
            iteration,
            time,
            values,
        })
    }

    #[instrument(level = "trace", name = "MainFrameDef::parse_inter", skip_all)]
    pub(crate) fn parse_inter(
        &self,
        data: &mut Reader,
        config: &Config,
        headers: &Headers,
        last: Option<&MainFrame>,
        last_last: Option<&MainFrame>,
        skipped_frames: u32,
    ) -> ParseResult<MainFrame> {
        let iteration = 1 + last.map_or(0, MainFrame::iteration) + skipped_frames;
        tracing::trace!(iteration);

        let time = {
            let raw = decode::variable_signed(data)?.into();

            if config.raw {
                tracing::trace!(time = raw);
                raw
            } else {
                let offset = last_last.map_or(0, |ll| last.unwrap().time - ll.time)
                    + last.map_or(0, MainFrame::time);

                let time = offset + raw;

                tracing::trace!(time, raw);
                time
            }
        };

        let mut fields = self.fields.iter().peekable();
        let mut values = Vec::with_capacity(self.fields.len());

        while let Some(field) = fields.next() {
            let encoding = field.encoding_inter;
            let extra = encoding.max_chunk_size() - 1;
            let extra = count_fields_with_same_encoding(&mut fields, extra, |&field| {
                field.encoding_inter == encoding
            });

            let mut new = encoding.decode(data, headers.version, extra)?;
            debug_assert_eq!(extra + 1, new.len());
            values.append(&mut new);
        }

        debug_assert_eq!(values.len(), self.fields.len());
        for (i, value) in values.iter_mut().enumerate() {
            let field = &self.fields[i];
            let raw = *value;

            let last = last.map(|l| l.values[i]);
            let last_last = last_last.map(|l| l.values[i]);

            if !config.raw {
                *value = field.predictor_inter.apply(
                    headers,
                    *value,
                    last,
                    last_last,
                    skipped_frames.into(),
                );
            }

            tracing::trace!(
                field = field.name,
                encoding = ?field.encoding_inter,
                predictor = ?field.predictor_inter,
                raw,
                value,
            );

            // TODO: check field.signed
        }

        Ok(MainFrame {
            iteration,
            time,
            values,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MainFieldDef<'data> {
    pub(crate) name: &'data str,
    predictor_intra: Predictor,
    predictor_inter: Predictor,
    encoding_intra: Encoding,
    encoding_inter: Encoding,
}

#[derive(Debug, Default)]
pub(crate) struct MainFrameDefBuilder<'data> {
    pub(crate) names: Option<&'data str>,
    pub(crate) predictors_intra: Option<&'data str>,
    pub(crate) predictors_inter: Option<&'data str>,
    pub(crate) encodings_intra: Option<&'data str>,
    pub(crate) encodings_inter: Option<&'data str>,
}

impl<'data> MainFrameDefBuilder<'data> {
    pub(crate) fn update(&mut self, kind: FrameKind, property: FrameProperty, value: &'data str) {
        let value = Some(value);

        match (kind, property) {
            (_, FrameProperty::Name) => self.names = value,

            (FrameKind::Intra, FrameProperty::Predictor) => self.predictors_intra = value,
            (FrameKind::Inter, FrameProperty::Predictor) => self.predictors_inter = value,

            (FrameKind::Intra, FrameProperty::Encoding) => self.encodings_intra = value,
            (FrameKind::Inter, FrameProperty::Encoding) => self.encodings_inter = value,
            _ => unreachable!(),
        }
    }

    pub(crate) fn parse(self) -> ParseResult<MainFrameDef<'data>> {
        let kind_intra = FrameKind::Intra;
        let kind_inter = FrameKind::Inter;

        let mut names = super::parse_names(kind_intra, self.names)?;
        let mut predictors_intra = super::parse_predictors(kind_intra, self.predictors_intra)?;
        let mut predictors_inter = super::parse_predictors(kind_inter, self.predictors_inter)?;
        let mut encodings_intra = super::parse_encodings(kind_intra, self.encodings_intra)?;
        let mut encodings_inter = super::parse_encodings(kind_inter, self.encodings_inter)?;

        let mut fields = names
            .by_ref()
            .zip(predictors_intra.by_ref().zip(predictors_inter.by_ref()))
            .zip(encodings_intra.by_ref().zip(encodings_inter.by_ref()))
            .map(
                |((name, (predictor_intra, predictor_inter)), (encoding_intra, encoding_inter))| {
                    Ok(MainFieldDef {
                        name,
                        predictor_intra: predictor_intra?,
                        predictor_inter: predictor_inter?,
                        encoding_intra: encoding_intra?,
                        encoding_inter: encoding_inter?,
                    })
                },
            );

        // TODO: improve errors
        let iteration = match fields.next() {
            Some(Ok(
                field @ MainFieldDef {
                    predictor_intra: Predictor::Zero,
                    predictor_inter: Predictor::Increment,
                    encoding_intra: Encoding::Variable,
                    encoding_inter: Encoding::Null,
                    ..
                },
            )) if field.name.to_ascii_lowercase() == "loopiteration" => field,
            _ => return Err(ParseError::Corrupted),
        };
        let time = match fields.next() {
            Some(Ok(
                field @ MainFieldDef {
                    predictor_intra: Predictor::Zero,
                    predictor_inter: Predictor::StraightLine,
                    encoding_intra: Encoding::Variable,
                    encoding_inter: Encoding::VariableSigned,
                    ..
                },
            )) if field.name.to_ascii_lowercase() == "time" => field,
            _ => return Err(ParseError::Corrupted),
        };
        let fields = fields.collect::<ParseResult<Vec<_>>>()?;

        assert!(
            names.next().is_none()
                && predictors_intra.next().is_none()
                && predictors_inter.next().is_none()
                && encodings_intra.next().is_none()
                && encodings_inter.next().is_none(),
            "all `Field *` headers must have the same number of elements"
        );

        Ok(MainFrameDef {
            iteration,
            time,
            fields,
        })
    }
}
