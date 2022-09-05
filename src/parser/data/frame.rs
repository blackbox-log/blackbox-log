use super::Headers;
use crate::{encoding, Reader};
use crate::{Encoding, FieldDef, FrameDef, ParseResult, Predictor};
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
    Event,
    Intra,
    Inter,
    Gps,
    GpsHome,
    Slow,
}

impl FrameKind {
    pub(crate) fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'E' => Some(Self::Event),
            b'I' => Some(Self::Intra),
            b'P' => Some(Self::Inter),
            b'G' => Some(Self::Gps),
            b'H' => Some(Self::GpsHome),
            b'S' => Some(Self::Slow),
            _ => None,
        }
    }
}

// Reason: unfinished
#[allow(dead_code)]
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
    pub(crate) fn parse(
        data: &mut Reader,
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
                        crate::byte_align(data);
                        values.push(encoding::read_ivar(data)?.into());
                        vec![field]
                    }
                    Encoding::UVar => {
                        crate::byte_align(data);
                        values.push(encoding::read_uvar(data)?.into());
                        vec![field]
                    }
                    Encoding::Negative14Bit => {
                        crate::byte_align(data);
                        values.push(encoding::read_negative_14_bit(data)?.into());
                        vec![field]
                    }
                    Encoding::U32EliasDelta => {
                        values.push(encoding::read_u32_elias_delta(data)?.into());
                        vec![field]
                    }
                    Encoding::I32EliasDelta => {
                        values.push(encoding::read_i32_elias_delta(data)?.into());
                        vec![field]
                    }
                    Encoding::Tagged32 => {
                        crate::byte_align(data);

                        let read_values = encoding::read_tagged_32(data)?.map(i64::from);

                        let fields = fields_with_same_encoding(frame_fields.by_ref(), field);
                        assert!(fields.len() <= read_values.len());

                        values.extend_from_slice(&read_values);

                        fields
                    }
                    Encoding::Tagged16 => {
                        crate::byte_align(data);

                        let read_values =
                            encoding::read_tagged_16(headers.version, data)?.map(i64::from);

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
                    other @ (Encoding::TaggedVar
                    | Encoding::U32EliasGamma
                    | Encoding::I32EliasGamma) => unimplemented!("{other:?}"),
                }
            };

            for (field, value) in read_fields.into_iter().zip(values.iter_mut().rev()) {
                *value = field.predictor.apply(*value);
                tracing::debug!(field = field.name, value);
            }
        }

        crate::byte_align(data);

        Ok(Self {
            kind: frame_def.kind,
            values,
        })
    }
}
