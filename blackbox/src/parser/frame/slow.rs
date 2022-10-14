use alloc::vec::Vec;
use core::iter;

use tracing::instrument;

use super::{count_fields_with_same_encoding, FrameKind, FrameProperty};
use crate::parser::{as_signed, Encoding, Headers, ParseError, ParseResult, Predictor, Reader};
use crate::units;

#[derive(Debug, Clone)]
pub struct SlowFrame {
    pub(crate) values: Vec<SlowValue>,
}

impl SlowFrame {
    pub(crate) fn iter(&self) -> impl Iterator<Item = SlowValue> + '_ {
        self.values.iter().copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlowValue {
    FlightMode(units::FlightMode),
    Unsigned(u32),
    Signed(i32),
}

impl SlowValue {
    fn new_unitless(value: u32, signed: bool) -> Self {
        if signed {
            Self::Signed(as_signed(value))
        } else {
            Self::Unsigned(value)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlowUnit {
    FlightMode,
    Unitless,
}

#[derive(Debug, Clone)]
pub(crate) struct SlowFrameDef<'data>(pub(crate) Vec<SlowFieldDef<'data>>);

impl<'data> SlowFrameDef<'data> {
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, SlowUnit)> {
        self.0.iter().map(|f| (f.name, f.unit))
    }

    pub(crate) fn builder() -> SlowFrameDefBuilder<'data> {
        SlowFrameDefBuilder::default()
    }

    #[instrument(level = "trace", name = "SlowFrameDef::parse", skip_all)]
    pub(crate) fn parse(&self, data: &mut Reader, headers: &Headers) -> ParseResult<SlowFrame> {
        let mut fields = self.0.iter().peekable();
        let mut raw = Vec::with_capacity(self.0.len());

        while let Some(field) = fields.next() {
            let encoding = field.encoding;
            let extra = encoding.max_chunk_size() - 1;
            let extra = count_fields_with_same_encoding(&mut fields, extra, |&field| {
                field.encoding == encoding
            });

            encoding.decode_into(data, headers.version, extra, &mut raw)?;
        }

        debug_assert_eq!(raw.len(), self.0.len());
        let mut values = Vec::with_capacity(raw.len());
        for i in 0..raw.len() {
            let field = &self.0[i];

            let value =
                field
                    .predictor
                    .apply(headers, raw[i], field.signed, &raw, None, None, 0)?;

            values.push(match field.unit {
                SlowUnit::FlightMode => {
                    SlowValue::FlightMode(units::FlightMode::new(value, headers))
                }
                SlowUnit::Unitless => SlowValue::new_unitless(value, field.signed),
            });

            tracing::trace!(
                field = field.name,
                encoding = ?field.encoding,
                predictor = ?field.predictor,
                raw = raw[i],
                value,
            );
        }

        Ok(SlowFrame { values })
    }

    pub(crate) fn default_frame(&self, headers: &Headers) -> SlowFrame {
        let mut i = 0;
        let values = iter::from_fn(|| {
            self.0.get(i).map(|def| {
                i += 1;

                match def.unit {
                    SlowUnit::FlightMode => {
                        SlowValue::FlightMode(units::FlightMode::new(0, headers))
                    }
                    SlowUnit::Unitless => SlowValue::new_unitless(0, def.signed),
                }
            })
        })
        .collect();

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
    pub(crate) fn update(&mut self, property: FrameProperty, value: &'data str) {
        let value = Some(value);

        match property {
            FrameProperty::Name => self.names = value,
            FrameProperty::Predictor => self.predictors = value,
            FrameProperty::Encoding => self.encodings = value,
            FrameProperty::Signed => self.signs = value,
        }
    }

    pub(crate) fn parse(self) -> ParseResult<SlowFrameDef<'data>> {
        let kind = FrameKind::Slow;

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
            .collect::<ParseResult<Vec<_>>>()?;

        if names.next().is_some()
            || predictors.next().is_some()
            || encodings.next().is_some()
            || signs.next().is_some()
        {
            tracing::error!("all `Field *` headers must have the same number of elements");
            return Err(ParseError::Corrupted);
        }

        Ok(SlowFrameDef(fields))
    }
}

fn unit_from_name(name: &str) -> SlowUnit {
    match name.to_ascii_lowercase().as_str() {
        "flightmodeflags" => SlowUnit::FlightMode,
        _ => SlowUnit::Unitless,
    }
}
