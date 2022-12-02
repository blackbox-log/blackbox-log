use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::iter;

use tracing::instrument;

use super::{read_field_values, DataFrameKind, DataFrameProperty, FrameKind, Unit};
use crate::data::FrameSync;
use crate::parser::{decode, to_base_field, Encoding, InternalResult};
use crate::predictor::{self, Predictor, PredictorContext};
use crate::units::prelude::*;
use crate::units::FromRaw;
use crate::utils::as_i32;
use crate::{Headers, HeadersParseError, HeadersParseResult, Reader};

#[derive(Debug, Clone)]
pub(crate) struct MainFrame {
    intra: bool,
    pub(crate) iteration: u32,
    pub(crate) time: u64,
    pub(crate) values: Vec<u32>,
}

impl MainFrame {
    pub(crate) fn parse(
        data: &mut Reader,
        kind: FrameKind,
        main_frames: &[FrameSync],
        headers: &Headers,
    ) -> InternalResult<Self> {
        let get_main_frame = |i| main_frames.get(i).map(|sync: &FrameSync| &sync.main);

        let current_idx = main_frames.len();
        let last = current_idx.checked_sub(1).and_then(get_main_frame);
        let main = &headers.main_frames;

        if kind == FrameKind::Data(DataFrameKind::Intra) {
            main.parse_intra(data, headers, last)
        } else {
            let last_last = current_idx.checked_sub(2).and_then(get_main_frame);
            let skipped = 0; // FIXME

            main.parse_inter(data, headers, last, last_last, skipped)
        }
    }

    pub(crate) fn get(&self, index: usize, headers: &Headers) -> Option<MainValue> {
        let value = match index {
            0 => MainValue::Unsigned(self.iteration),
            1 => MainValue::FrameTime(Time::from_raw(self.time, headers)),
            _ => {
                let index = index - 2;
                let def = headers.main_frames.fields.get(index)?;
                let raw = self.values[index];
                match def.unit {
                    MainUnit::Amperage => {
                        debug_assert!(def.signed);
                        let raw = as_i32(raw);
                        MainValue::Amperage(ElectricCurrent::from_raw(raw, headers))
                    }
                    MainUnit::Voltage => {
                        debug_assert!(!def.signed);
                        MainValue::Voltage(ElectricPotential::from_raw(raw, headers))
                    }
                    MainUnit::Acceleration => {
                        debug_assert!(def.signed);
                        let raw = as_i32(raw);
                        MainValue::Acceleration(Acceleration::from_raw(raw, headers))
                    }
                    MainUnit::Rotation => {
                        debug_assert!(def.signed);
                        let raw = as_i32(raw);
                        MainValue::Rotation(AngularVelocity::from_raw(raw, headers))
                    }
                    MainUnit::Unitless => MainValue::new_unitless(raw, def.signed),
                    MainUnit::FrameTime => unreachable!(),
                }
            }
        };

        Some(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MainValue {
    FrameTime(Time),
    Amperage(ElectricCurrent),
    Voltage(ElectricPotential),
    Acceleration(Acceleration),
    Rotation(AngularVelocity),
    Unsigned(u32),
    Signed(i32),
}

impl MainValue {
    const fn new_unitless(value: u32, signed: bool) -> Self {
        if signed {
            Self::Signed(as_i32(value))
        } else {
            Self::Unsigned(value)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MainUnit {
    FrameTime,
    Amperage,
    Voltage,
    Acceleration,
    Rotation,
    Unitless,
}

#[derive(Debug, Clone)]
pub(crate) struct MainFrameDef<'data> {
    pub(crate) iteration: MainFieldDef<'data>,
    pub(crate) time: MainFieldDef<'data>,
    pub(crate) fields: Vec<MainFieldDef<'data>>,

    index_motor_0: Option<usize>,
}

impl<'data> MainFrameDef<'data> {
    pub(crate) fn len(&self) -> usize {
        2 + self.fields.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<(&str, MainUnit)> {
        let field = match index {
            0 => &self.iteration,
            1 => &self.time,
            _ => self.fields.get(index - 2)?,
        };

        Some((field.name, field.unit))
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, MainUnit)> {
        let Self {
            iteration,
            time,
            fields,
            ..
        } = self;

        iter::once((iteration.name, iteration.unit))
            .chain(iter::once((time.name, time.unit)))
            .chain(fields.iter().map(|f| (f.name, f.unit)))
    }

    pub(crate) fn builder() -> MainFrameDefBuilder<'data> {
        MainFrameDefBuilder::default()
    }

    pub(crate) const fn has_motor_0(&self) -> bool {
        self.index_motor_0.is_some()
    }

    /// # Panics
    ///
    /// Panics if there is no `motor[0]` field in the frame
    pub(crate) fn get_motor_0_from(&self, frame: &[u32]) -> u32 {
        frame[self.index_motor_0.unwrap()]
    }

    pub(crate) fn validate(
        &self,
        check_predictor: impl Fn(&'data str, Predictor) -> HeadersParseResult<()>,
        check_unit: impl Fn(&'data str, Unit) -> HeadersParseResult<()>,
    ) -> HeadersParseResult<()> {
        for MainFieldDef {
            name,
            predictor_intra,
            predictor_inter,
            unit,
            ..
        } in iter::once(&self.iteration)
            .chain(iter::once(&self.time))
            .chain(self.fields.iter())
        {
            check_predictor(name, *predictor_intra)?;
            check_predictor(name, *predictor_inter)?;
            check_unit(name, Unit::from(*unit))?;
        }

        Ok(())
    }

    #[instrument(level = "trace", skip_all)]
    pub(crate) fn parse_intra(
        &self,
        data: &mut Reader,
        headers: &Headers,
        last: Option<&MainFrame>,
    ) -> InternalResult<MainFrame> {
        let iteration = decode::variable(data)?;
        tracing::trace!(iteration);
        let time = decode::variable(data)?.into();
        tracing::trace!(time);

        let raw = read_field_values(data, &self.fields, |f| f.encoding_intra)?;

        let mut ctx = PredictorContext::new(headers);
        let mut values = Vec::with_capacity(raw.len());

        for (i, field) in self.fields.iter().enumerate() {
            let raw = raw[i];
            let signed = field.encoding_intra.is_signed();

            ctx.set_last(last.map(|l| l.values[i]));

            trace_field!(pre, field = field, enc = field.encoding_intra, raw = raw);

            let value = field
                .predictor_intra
                .apply(raw, signed, Some(&values), &ctx);
            values.push(value);

            trace_field!(
                post,
                field = field,
                pred = field.predictor_intra,
                final = value
            );
        }

        Ok(MainFrame {
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
        last: Option<&MainFrame>,
        last_last: Option<&MainFrame>,
        skipped_frames: u32,
    ) -> InternalResult<MainFrame> {
        let iteration = 1 + last.map_or(0, |f| f.iteration) + skipped_frames;
        tracing::trace!(iteration);

        let time = {
            let last_last = last
                .filter(|f| !f.intra)
                .and_then(|_| last_last.map(|f| f.time));

            let time = predictor::straight_line(last.map(|f| f.time), last_last);
            let offset = decode::variable_signed(data)?;

            // TODO (rust 1.66): replace with time.saturating_add_unsigned(offset.into())
            let add = offset > 0;
            let offset = offset.unsigned_abs().into();
            let time = if add {
                time.saturating_add(offset)
            } else {
                time.saturating_sub(offset)
            };

            tracing::trace!(time, offset);
            time
        };

        let raw = read_field_values(data, &self.fields, |f| f.encoding_inter)?;

        let mut ctx = PredictorContext::with_skipped(headers, skipped_frames);
        let mut values = Vec::with_capacity(raw.len());

        for (i, field) in self.fields.iter().enumerate() {
            let raw = raw[i];
            let signed = field.encoding_inter.is_signed();

            ctx.set_last_2(last.map(|l| l.values[i]), last_last.map(|l| l.values[i]));

            trace_field!(pre, field = field, enc = field.encoding_inter, raw = raw);

            let value = field
                .predictor_inter
                .apply(raw, signed, Some(&values), &ctx);
            values.push(value);

            trace_field!(
                post,
                field = field,
                pred = field.predictor_inter,
                final = value
            );
        }

        Ok(MainFrame {
            intra: false,
            iteration,
            time,
            values,
        })
    }
}

#[cfg(fuzzing)]
impl Default for MainFrameDef<'static> {
    fn default() -> Self {
        let default_def = MainFieldDef {
            name: "",
            predictor_intra: Predictor::Zero,
            predictor_inter: Predictor::Zero,
            encoding_intra: Encoding::Null,
            encoding_inter: Encoding::Null,
            signed: false,
            unit: MainUnit::Unitless,
        };

        Self {
            iteration: default_def.clone(),
            time: default_def,
            fields: Vec::new(),

            index_motor_0: None,
        }
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

    pub(crate) fn parse(self) -> HeadersParseResult<MainFrameDef<'data>> {
        let kind_intra = DataFrameKind::Intra;
        let kind_inter = DataFrameKind::Inter;

        let mut names = super::parse_names(kind_intra, self.names)?;
        let mut predictors_intra = super::parse_predictors(kind_intra, self.predictors_intra)?;
        let mut predictors_inter = super::parse_predictors(kind_inter, self.predictors_inter)?;
        let mut encodings_intra = super::parse_encodings(kind_intra, self.encodings_intra)?;
        let mut encodings_inter = super::parse_encodings(kind_inter, self.encodings_inter)?;
        let mut signs = super::parse_signs(kind_intra, self.signs)?;

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

        let Some(iteration @ MainFieldDef {
            name: "loopIteration",
            predictor_intra: Predictor::Zero,
            predictor_inter: Predictor::Increment,
            encoding_intra: Encoding::Variable,
            encoding_inter: Encoding::Null,
            ..
        }) = fields.next().transpose()? else {
            return Err(HeadersParseError::MissingField {
                frame: DataFrameKind::Intra,
                field: "loopIteration".to_owned()
            });
        };

        let Some(time @ MainFieldDef {
            name: "time",
            predictor_intra: Predictor::Zero,
            predictor_inter: Predictor::StraightLine,
            encoding_intra: Encoding::Variable,
            encoding_inter: Encoding::VariableSigned,
            ..
        }) = fields.next().transpose()? else {
            return Err(HeadersParseError::MissingField {
                frame: DataFrameKind::Intra,
                field: "time".to_owned()
            });
        };

        let fields = fields.collect::<Result<Vec<_>, _>>()?;

        if names.next().is_some()
            || predictors_intra.next().is_some()
            || predictors_inter.next().is_some()
            || encodings_intra.next().is_some()
            || encodings_inter.next().is_some()
            || signs.next().is_some()
        {
            tracing::warn!(
                "not all interframe & intraframe definition headers are of equal length"
            );
        }

        let index_motor_0 = fields.iter().position(|f| f.name == "motor[0]");

        Ok(MainFrameDef {
            iteration,
            time,
            fields,

            index_motor_0,
        })
    }
}

fn unit_from_name(name: &str) -> MainUnit {
    match to_base_field(name) {
        "time" => MainUnit::FrameTime,
        "vbat" | "vbatLatest" => MainUnit::Voltage,
        "amperageLatest" => MainUnit::Amperage,
        "accSmooth" => MainUnit::Acceleration,
        "gyroADC" => MainUnit::Rotation,
        _ => MainUnit::Unitless,
    }
}
