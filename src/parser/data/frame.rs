use super::Headers;
use crate::encoding;
use crate::{Encoding, FieldDef, FrameDef, ParseResult, Predictor};
use biterator::Biterator;
use std::io::Read;
use std::iter::Peekable;
use tracing::instrument;

fn fields_with_same_encoding<'a, I>(
    fields: &mut Peekable<I>,
    field: &'a FieldDef,
) -> Vec<&'a FieldDef>
where
    I: Iterator<Item = &'a FieldDef>,
{
    let mut results = vec![field];

    while let Some(next) = fields.next_if(|i| i.encoding == field.encoding) {
        results.push(next);
    }

    results
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameKind {
    Intra, // I
    // Inter, // P
    // Gps, // G
    // GpsHome, // H
    // Event, // E
    Slow, // S
}

#[derive(Debug, Clone)]
pub struct Frame {
    kind: FrameKind,
    values: Vec<i64>,
}

impl Frame {
    #[instrument(
        level = "debug",
        name = "Frame::parse",
        skip_all,
        fields(frame_type = ?frame_def.kind)
    )]
    pub(crate) fn parse<R: Read>(
        log: &mut Biterator<R>,
        headers: &Headers,
        frame_def: &FrameDef,
    ) -> ParseResult<Self> {
        let mut frame_fields = (&frame_def.fields).iter().peekable();
        let mut values: Vec<i64> = Vec::with_capacity(frame_def.fields.len());

        while let Some(field) = frame_fields.next() {
            let read_fields = if field.predictor == Predictor::Increment {
                todo!("Predictor::Increment")
            } else {
                match field.encoding {
                    Encoding::IVar => {
                        values.push(encoding::read_ivar(log)?.into());
                        vec![field]
                    }
                    Encoding::UVar => {
                        values.push(encoding::read_uvar(log)?.into());
                        vec![field]
                    }
                    Encoding::Negative14Bit => {
                        values.push(encoding::read_negative_14_bit(log)?.into());
                        vec![field]
                    }
                    Encoding::U32EliasDelta => {
                        values.push(encoding::read_u32_elias_delta(log)?.into());
                        vec![field]
                    }
                    Encoding::I32EliasDelta => {
                        values.push(encoding::read_i32_elias_delta(log)?.into());
                        vec![field]
                    }
                    Encoding::Tagged32 => {
                        let read_values = encoding::read_tagged_32(log)?.map(i64::from);

                        let fields = fields_with_same_encoding(frame_fields.by_ref(), field);
                        assert!(fields.len() <= read_values.len());

                        values.extend_from_slice(&read_values);

                        fields
                    }
                    Encoding::Tagged16 => {
                        let read_values =
                            encoding::read_tagged_16(headers.version, log)?.map(i64::from);

                        let fields = fields_with_same_encoding(frame_fields.by_ref(), field);
                        assert!(fields.len() <= read_values.len());

                        values.extend_from_slice(&read_values);

                        fields
                    }
                    Encoding::Null => {
                        // TODO: check if prediction needs to be applied
                        values.push(0);
                        vec![field]
                    }
                    other => unimplemented!("{other:?}"),
                }
            };

            for (i, field) in read_fields.into_iter().enumerate() {
                let value = values.len() - i - 1;
                let value = values.get_mut(value).unwrap();

                *value = field.predictor.apply(*value);

                tracing::debug!(field = field.name, value);
            }
        }

        log.byte_align();

        Ok(Self {
            kind: frame_def.kind,
            values,
        })
    }
}
