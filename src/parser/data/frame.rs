use crate::parser::headers::FrameDef;
use crate::parser::{decode, ParseError};
use crate::parser::{Config, DataFrameKind, Encoding, FieldDef, Headers, ParseResult, Predictor};
use crate::Reader;
use std::iter::Peekable;
use tracing::instrument;

fn check_next_encodings_match<'a, I>(
    fields: &mut Peekable<I>,
    extra: usize,
    encoding: Encoding,
) -> ParseResult<()>
where
    I: Iterator<Item = (usize, &'a FieldDef)>,
{
    let all_match = fields
        .take(extra)
        .all(|(_, field)| field.encoding() == encoding);

    if all_match {
        Ok(())
    } else {
        Err(ParseError::Corrupted)
    }
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
        skipped_frames: usize,
    ) -> ParseResult<Self> {
        let skipped_frames = i64::try_from(skipped_frames).unwrap();

        let mut frame_fields = frame_def.iter().enumerate().peekable();
        let mut values: Vec<i64> = Vec::with_capacity(frame_def.len());

        while let Some((i, field)) = frame_fields.next() {
            debug_assert_eq!(i, values.len());
            let mut extra_fields = 0;

            if field.predictor() == Predictor::Increment {
                let mut value = 1 + skipped_frames;

                if let Some(last) = last {
                    value += last.values[i];
                }

                values.push(value);
            } else {
                let encoding = field.encoding();

                if !matches!(
                    encoding,
                    Encoding::Tagged16 | Encoding::Tagged32 | Encoding::Null
                ) {
                    crate::byte_align(data);
                }

                match encoding {
                    Encoding::UVar => values.push(decode::variable(data)?.into()),
                    Encoding::IVar => values.push(decode::variable_signed(data)?.into()),

                    Encoding::Negative14Bit => values.push(decode::negative_14_bit(data)?.into()),

                    Encoding::U32EliasDelta => values.push(decode::elias_delta(data)?.into()),
                    Encoding::I32EliasDelta => {
                        values.push(decode::elias_delta_signed(data)?.into());
                    }

                    Encoding::Tagged32 => {
                        let read_values = decode::tagged_32(data)?.map(i64::from);

                        extra_fields = read_values.len() - 1;
                        check_next_encodings_match(frame_fields.by_ref(), extra_fields, encoding)?;

                        values.extend_from_slice(&read_values);
                    }

                    Encoding::Tagged16 => {
                        let read_values = decode::tagged_16(headers.version, data)?.map(i64::from);

                        extra_fields = read_values.len() - 1;
                        check_next_encodings_match(frame_fields.by_ref(), extra_fields, encoding)?;

                        values.extend_from_slice(&read_values);
                    }

                    Encoding::Null => values.push(0),

                    Encoding::TaggedVar | Encoding::U32EliasGamma | Encoding::I32EliasGamma => {
                        unimplemented!("{encoding:?}")
                    }
                }

                for i in i..=(i + extra_fields) {
                    let field = &frame_def[i];
                    let value = &mut values[i];

                    tracing::trace!(
                        field = field.name(),
                        value,
                        encoding = ?field.encoding(),
                        predictor = ?field.predictor()
                    );

                    let last = last.map(|l| l.values[i]);
                    let last_last = last_last.map(|l| l.values[i]);

                    if !config.raw {
                        *value = field.predictor().apply(headers, *value, last, last_last);
                    }

                    tracing::debug!(field = field.name(), value);

                    // TODO: check field.signed
                }
            }
        }

        crate::byte_align(data);

        Ok(Self {
            kind: frame_def.kind(),
            values,
        })
    }
}
