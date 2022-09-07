use crate::parser::decoders;
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
            let extra_fields = if field.predictor() == Predictor::Increment {
                todo!("Predictor::Increment")
            } else {
                match field.encoding() {
                    Encoding::IVar => {
                        crate::byte_align(data);
                        values.push(decoders::read_ivar(data)?.into());
                        0
                    }
                    Encoding::UVar => {
                        crate::byte_align(data);
                        values.push(decoders::read_uvar(data)?.into());
                        0
                    }
                    Encoding::Negative14Bit => {
                        crate::byte_align(data);
                        values.push(decoders::read_negative_14_bit(data)?.into());
                        0
                    }
                    Encoding::U32EliasDelta => {
                        values.push(decoders::read_u32_elias_delta(data)?.into());
                        0
                    }
                    Encoding::I32EliasDelta => {
                        values.push(decoders::read_i32_elias_delta(data)?.into());
                        0
                    }
                    Encoding::Tagged32 => {
                        crate::byte_align(data);

                        let read_values = decoders::read_tagged_32(data)?.map(i64::from);

                        let fields =
                            fields_with_same_encoding(frame_fields.by_ref(), field.encoding());
                        assert!(fields <= read_values.len());

                        values.extend_from_slice(&read_values);

                        fields
                    }
                    Encoding::Tagged16 => {
                        crate::byte_align(data);

                        let read_values =
                            decoders::read_tagged_16(headers.version, data)?.map(i64::from);

                        let fields =
                            fields_with_same_encoding(frame_fields.by_ref(), field.encoding());
                        assert!(fields <= read_values.len());

                        values.extend_from_slice(&read_values);

                        fields
                    }
                    Encoding::Null => {
                        // TODO: check if prediction needs to be applied
                        values.push(0);
                        0
                    }
                    other @ (Encoding::TaggedVar
                    | Encoding::U32EliasGamma
                    | Encoding::I32EliasGamma) => unimplemented!("{other:?}"),
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
