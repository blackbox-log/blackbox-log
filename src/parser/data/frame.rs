use crate::parser::decode;
use crate::parser::headers::FrameDef;
use crate::parser::{Config, DataFrameKind, Encoding, FieldDef, Headers, ParseResult, Predictor};
use crate::Reader;
use std::iter::Peekable;
use tracing::instrument;

fn fields_with_same_encoding<'a, I>(fields: &mut Peekable<I>, encoding: Encoding) -> usize
where
    I: Iterator<Item = (usize, &'a FieldDef)>,
{
    fields
        .take_while(|&(_, field)| field.encoding() == encoding)
        .count()
}

// Reason: unfinished
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Frame {
    kind: DataFrameKind,
    values: Vec<i64>,
}

impl Frame {
    #[instrument(
        level = "debug",
        name = "Frame::parse",
        skip_all,
        fields(frame_type = ?frame_def.kind())
    )]
    pub(crate) fn parse(
        data: &mut Reader,
        config: &Config,
        headers: &Headers,
        frame_def: &FrameDef,
        last: Option<&Frame>,
        last_last: Option<&Frame>,
    ) -> ParseResult<Self> {
        let mut frame_fields = frame_def.iter().enumerate().peekable();
        let mut values: Vec<i64> = Vec::with_capacity(frame_def.len());

        while let Some((i, field)) = frame_fields.next() {
            let mut extra_fields = 0;

            if field.predictor() == Predictor::Increment {
                todo!("Predictor::Increment")
            } else {
                let encoding = field.encoding();

                if !matches!(
                    encoding,
                    Encoding::Tagged16 | Encoding::Tagged32 | Encoding::Null
                ) {
                    crate::byte_align(data);
                }

                match encoding {
                    Encoding::IVar => values.push(decode::variable_signed(data)?.into()),
                    Encoding::UVar => values.push(decode::variable(data)?.into()),
                    Encoding::Negative14Bit => values.push(decode::negative_14_bit(data)?.into()),
                    Encoding::U32EliasDelta => values.push(decode::elias_delta(data)?.into()),
                    Encoding::I32EliasDelta => {
                        values.push(decode::elias_delta_signed(data)?.into());
                    }
                    Encoding::Tagged32 => {
                        let read_values = decode::tagged_32(data)?.map(i64::from);

                        let fields = fields_with_same_encoding(frame_fields.by_ref(), encoding);
                        assert!(fields <= read_values.len());
                        extra_fields = fields;

                        values.extend_from_slice(&read_values);
                    }
                    Encoding::Tagged16 => {
                        let read_values = decode::tagged_16(headers.version, data)?.map(i64::from);

                        let fields = fields_with_same_encoding(frame_fields.by_ref(), encoding);
                        assert!(fields <= read_values.len());
                        extra_fields = fields;

                        values.extend_from_slice(&read_values);
                    }
                    Encoding::Null => values.push(0),

                    Encoding::TaggedVar | Encoding::U32EliasGamma | Encoding::I32EliasGamma => {
                        unimplemented!("{encoding:?}")
                    }
                }
            };

            for i in i..=(i + extra_fields) {
                let field = &frame_def[i];
                let value = &mut values[i];
                let last = last.map(|l| l.values[i]);
                let last_last = last_last.map(|l| l.values[i]);

                if !config.raw {
                    *value = field.predictor().apply(headers, *value, last, last_last);
                }

                tracing::debug!(field = field.name(), value);

                // TODO: check field.signed
            }
        }

        crate::byte_align(data);

        Ok(Self {
            kind: frame_def.kind(),
            values,
        })
    }
}
