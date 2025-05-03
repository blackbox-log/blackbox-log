use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use tracing::instrument;

use super::{MainUnit, RawMainFrame};
use crate::frame::{self, DataFrameKind, DataFrameProperty, FieldDef, FieldDefDetails, FrameDef};
use crate::headers::{ParseError, ParseResult};
use crate::parser::{decode, Encoding, InternalResult};
use crate::predictor::{Predictor, PredictorContext};
use crate::utils::to_base_field;
use crate::{Headers, Reader, Unit};

/// The parsed frame definition for main frames.
#[derive(Debug, Clone)]
pub struct MainFrameDef<'data> {
    iteration: MainFieldDef<'data>,
    pub(super) fields: Vec<MainFieldDef<'data>>,
    pub(crate) index_motor_0: Option<usize>,
}

impl frame::seal::Sealed for MainFrameDef<'_> {}

impl<'data> FrameDef<'data> for MainFrameDef<'data> {
    type Unit = MainUnit;

    #[inline]
    fn len(&self) -> usize {
        // Plus loopIteration
        self.fields.len() + 1
    }

    fn get<'a>(&'a self, index: usize) -> Option<FieldDef<'data, Self::Unit>>
    where
        'data: 'a,
    {
        let field = if index == 0 {
            Some(&self.iteration)
        } else {
            self.fields.get(index - 1)
        };

        field.map(
            |&MainFieldDef {
                 name, signed, unit, ..
             }| FieldDef { name, unit, signed },
        )
    }
}

impl<'data> MainFrameDef<'data> {
    pub(crate) fn builder() -> MainFrameDefBuilder<'data> {
        MainFrameDefBuilder::default()
    }

    /// # Panics
    ///
    /// Panics if there is no `motor[0]` field in the frame
    pub(crate) fn get_motor_0_from(&self, frame: &[u32]) -> u32 {
        frame[self.index_motor_0.unwrap()]
    }

    pub(crate) fn validate(
        &self,
        check_predictor: impl Fn(DataFrameKind, &'data str, Predictor, usize) -> ParseResult<()>,
        check_unit: impl Fn(DataFrameKind, &'data str, Unit) -> ParseResult<()>,
    ) -> ParseResult<()> {
        for (
            i,
            MainFieldDef {
                name,
                predictor_intra,
                predictor_inter,
                unit,
                ..
            },
        ) in self.fields.iter().enumerate()
        {
            check_predictor(DataFrameKind::Intra, name, *predictor_intra, i)?;
            check_predictor(DataFrameKind::Inter, name, *predictor_inter, i)?;
            check_unit(DataFrameKind::Intra, name, Unit::from(*unit))?;
        }

        Ok(())
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn parse_intra(
        &self,
        data: &mut Reader,
        headers: &Headers,
        last: Option<&RawMainFrame>,
    ) -> InternalResult<RawMainFrame> {
        fn get_update_ctx(
            last: Option<&'_ RawMainFrame>,
        ) -> impl Fn(&mut PredictorContext, usize) + '_ {
            move |ctx, i| ctx.set_last(last.map(|l| l.values[i]))
        }

        let iteration = decode::variable(data)?;
        tracing::trace!(iteration);
        let time = decode::variable(data)?.into();
        tracing::trace!(time);

        let values = frame::parse_impl(
            PredictorContext::new(headers),
            &frame::read_field_values(data, &self.fields, |f| f.encoding_intra)?,
            self.fields.iter().map(IntraFieldDef),
            get_update_ctx(last),
        );

        Ok(RawMainFrame {
            intra: true,
            iteration,
            time,
            values,
        })
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn parse_inter(
        &self,
        data: &mut Reader,
        headers: &Headers,
        last: Option<&RawMainFrame>,
        last_last: Option<&RawMainFrame>,
        skipped_frames: u32,
    ) -> InternalResult<RawMainFrame> {
        fn get_update_ctx<'a>(
            last: Option<&'a RawMainFrame>,
            last_last: Option<&'a RawMainFrame>,
        ) -> impl Fn(&mut PredictorContext<'_, '_>, usize) + 'a {
            move |ctx, i| {
                ctx.set_last_2(last.map(|l| l.values[i]), last_last.map(|l| l.values[i]));
            }
        }

        let iteration = 1 + last.map_or(0, |f| f.iteration) + skipped_frames;
        tracing::trace!(iteration);

        let time = {
            // Get the time from last_last if last was an interframe
            let last_last = last
                .filter(|f| !f.intra)
                .and_then(|_| last_last.map(|f| f.time));

            let time: u64 = todo!();
            let offset = decode::variable_signed(data)?;
            let time = time.saturating_add_signed(offset.into());

            tracing::trace!(time, offset);
            time
        };

        let values = frame::parse_impl(
            PredictorContext::with_skipped(headers, skipped_frames),
            &frame::read_field_values(data, &self.fields, |f| f.encoding_inter)?,
            self.fields.iter().map(InterFieldDef),
            get_update_ctx(last, last_last),
        );

        Ok(RawMainFrame {
            intra: false,
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
    pub(crate) signed: bool,
    pub(crate) unit: MainUnit,
}

#[derive(Debug)]
struct InterFieldDef<'a, 'data>(&'a MainFieldDef<'data>);

impl<'data> FieldDefDetails<'data> for InterFieldDef<'_, 'data> {
    fn name(&self) -> &'data str {
        self.0.name
    }

    fn predictor(&self) -> Predictor {
        self.0.predictor_inter
    }

    fn encoding(&self) -> Encoding {
        self.0.encoding_inter
    }

    fn signed(&self) -> bool {
        self.0.signed
    }
}

#[derive(Debug)]
struct IntraFieldDef<'a, 'data>(&'a MainFieldDef<'data>);

impl<'data> FieldDefDetails<'data> for IntraFieldDef<'_, 'data> {
    fn name(&self) -> &'data str {
        self.0.name
    }

    fn predictor(&self) -> Predictor {
        self.0.predictor_intra
    }

    fn encoding(&self) -> Encoding {
        self.0.encoding_intra
    }

    fn signed(&self) -> bool {
        self.0.signed
    }
}

#[derive(Debug, Default)]
pub(crate) struct MainFrameDefBuilder<'data> {
    names: Option<&'data str>,
    predictors_intra: Option<&'data str>,
    predictors_inter: Option<&'data str>,
    encodings_intra: Option<&'data str>,
    encodings_inter: Option<&'data str>,
    signs: Option<&'data str>,
}

impl<'data> MainFrameDefBuilder<'data> {
    pub(crate) fn update(
        &mut self,
        kind: DataFrameKind,
        property: DataFrameProperty,
        value: &'data str,
    ) {
        let value = Some(value);

        match (kind, property) {
            (_, DataFrameProperty::Name) => self.names = value,
            (_, DataFrameProperty::Signed) => self.signs = value,

            (DataFrameKind::Intra, DataFrameProperty::Predictor) => self.predictors_intra = value,
            (DataFrameKind::Inter, DataFrameProperty::Predictor) => self.predictors_inter = value,

            (DataFrameKind::Intra, DataFrameProperty::Encoding) => self.encodings_intra = value,
            (DataFrameKind::Inter, DataFrameProperty::Encoding) => self.encodings_inter = value,
            _ => unreachable!(),
        }
    }

    pub(crate) fn parse(self) -> ParseResult<MainFrameDef<'data>> {
        let kind_intra = DataFrameKind::Intra;
        let kind_inter = DataFrameKind::Inter;

        let mut names = frame::parse_names(kind_intra, self.names)?;
        let mut predictors_intra = frame::parse_predictors(kind_intra, self.predictors_intra)?;
        let mut predictors_inter = frame::parse_predictors(kind_inter, self.predictors_inter)?;
        let mut encodings_intra = frame::parse_encodings(kind_intra, self.encodings_intra)?;
        let mut encodings_inter = frame::parse_encodings(kind_inter, self.encodings_inter)?;
        let mut signs = frame::parse_signs(kind_intra, self.signs)?;

        let mut fields = (names.by_ref().zip(signs.by_ref()))
            .zip(predictors_intra.by_ref().zip(predictors_inter.by_ref()))
            .zip(encodings_intra.by_ref().zip(encodings_inter.by_ref()))
            .map(
                |(
                    ((name, signed), (predictor_intra, predictor_inter)),
                    (encoding_intra, encoding_inter),
                )| {
                    Ok(MainFieldDef {
                        name,
                        predictor_intra: predictor_intra?,
                        predictor_inter: predictor_inter?,
                        encoding_intra: encoding_intra?,
                        encoding_inter: encoding_inter?,
                        signed,
                        unit: unit_from_name(name),
                    })
                },
            );

        let Some(
            iteration @ MainFieldDef {
                name: "loopIteration",
                predictor_intra: Predictor::Zero,
                predictor_inter: Predictor::Increment,
                encoding_intra: Encoding::Variable,
                encoding_inter: Encoding::Null,
                ..
            },
        ) = fields.next().transpose()?
        else {
            return Err(ParseError::MissingField {
                frame: DataFrameKind::Intra,
                field: "loopIteration".to_owned(),
            });
        };

        if !matches!(
            fields.next().transpose()?,
            Some(MainFieldDef {
                name: "time",
                predictor_intra: Predictor::Zero,
                predictor_inter: Predictor::StraightLine,
                encoding_intra: Encoding::Variable,
                encoding_inter: Encoding::VariableSigned,
                ..
            })
        ) {
            return Err(ParseError::MissingField {
                frame: DataFrameKind::Intra,
                field: "time".to_owned(),
            });
        }

        let fields = fields.collect::<Result<Vec<_>, _>>()?;

        if names.next().is_some()
            || predictors_intra.next().is_some()
            || encodings_intra.next().is_some()
            || signs.next().is_some()
        {
            tracing::error!("not all intraframe definition headers are of equal length");
            return Err(ParseError::MalformedFrameDef(DataFrameKind::Intra));
        }

        if predictors_inter.next().is_some() || encodings_inter.next().is_some() {
            tracing::error!("not all interframe definition headers are of equal length");
            return Err(ParseError::MalformedFrameDef(DataFrameKind::Inter));
        }

        let index_motor_0 = fields.iter().position(|f| f.name == "motor[0]");

        Ok(MainFrameDef {
            iteration,
            fields,
            index_motor_0,
        })
    }
}

fn unit_from_name(name: &str) -> MainUnit {
    match to_base_field(name) {
        "vbat" | "vbatLatest" => MainUnit::Voltage,
        "amperageLatest" => MainUnit::Amperage,
        "accSmooth" => MainUnit::Acceleration,
        "gyroADC" => MainUnit::Rotation,
        _ => MainUnit::Unitless,
    }
}
