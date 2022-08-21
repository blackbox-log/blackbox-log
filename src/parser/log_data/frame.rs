use super::Headers;
use crate::encoding;
use crate::{Encoding, FieldDef, FrameDef, ParseResult, Predictor};
use biterator::Biterator;
use std::io::Read;
use std::iter::Peekable;

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

#[derive(Debug, Clone)]
pub struct Frame {
    values: Vec<i64>,
}

impl Frame {
    pub(crate) fn parse<R: Read>(
        log: &mut Biterator<R>,
        headers: &Headers,
        frame_def: &FrameDef,
    ) -> ParseResult<Self> {
        let mut fields = (&frame_def.0).iter().peekable();
        let mut values: Vec<i64> = Vec::with_capacity(frame_def.0.len());

        while let Some(field) = fields.next() {
            let read_fields = if field.predictor == Predictor::Increment {
                todo!("Predictor::Increment")
            } else {
                match field.encoding {
                    Encoding::IVar => {
                        values.push(encoding::read_ivar(log).unwrap().into());
                        vec![field]
                    }
                    Encoding::UVar => {
                        values.push(encoding::read_uvar(log).unwrap().into());
                        vec![field]
                    }
                    Encoding::Negative14Bit => {
                        values.push(encoding::read_negative_14_bit(log).into());
                        vec![field]
                    }
                    Encoding::U32EliasDelta => {
                        values.push(encoding::read_u32_elias_delta(log).unwrap().into());
                        vec![field]
                    }
                    Encoding::I32EliasDelta => {
                        values.push(encoding::read_i32_elias_delta(log).unwrap().into());
                        vec![field]
                    }
                    Encoding::Tagged16 => {
                        let read_values = encoding::read_tagged_16(headers.version, log);

                        let fields = fields_with_same_encoding(fields.by_ref(), field);
                        assert!(fields.len() <= read_values.len());

                        values.extend(read_values.iter().map(|x| i64::from(*x)).take(fields.len()));

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

                // if field.name == "flightModeFlags" {
                //     eprintln!(
                //         "flightModeFlags: {:?}",
                //         betaflight::FlightModeFlags::new((*value).try_into().unwrap()).to_modes()
                //     );
                // } else {
                //     eprintln!("{}: {value:?}", field.name);
                // }
            }
        }

        Ok(Self { values })
    }
}
