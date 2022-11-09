use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::iter;

use tracing::instrument;

use super::{read_field_values, DataFrameKind, DataFrameProperty, Unit};
use crate::parser::{
    as_signed, decode, predictor, to_base_field, Encoding, FrameKind, Headers, InternalResult,
    ParseError, ParseResult, Predictor, Reader,
};
use crate::units;

macro_rules! trace_field {
    (_impl pre $field:expr, $enc:expr, $signed:expr, $raw:expr) => {
        tracing::trace!(
            field = $field.name,
            encoding = ?$enc,
            signed_encoding = $signed,
            raw = $raw,
        );
    };
    (_impl post $field:expr, $pred:expr, $signed:expr, $final:expr) => {
        tracing::trace!(
            field = $field.name,
            predictor = ?$pred,
            signed = $signed,
            value = $final,
        );
    };

    (pre, field = $field:expr, enc = $enc:expr, raw = $raw:expr $(,)?) => {
        if $enc.is_signed() {
            trace_field!(_impl pre $field, $enc, $enc.is_signed(), crate::parser::as_signed($raw));
        } else {
            trace_field!(_impl pre $field, $enc, $enc.is_signed(), $raw);
        }
    };
    (post, field = $field:expr, pred = $pred:expr, final = $final:expr $(,)?) => {
        if $field.signed {
            trace_field!(_impl post $field, $pred, $field.signed, crate::parser::as_signed($final));
        } else {
            trace_field!(_impl post $field, $pred, $field.signed, $final);
        }
    };
}

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
        main_frames: &[(MainFrame, usize)],
        headers: &Headers,
    ) -> InternalResult<Self> {
        let get_main_frame = |i| main_frames.get(i).map(|(frame, _)| frame);

        let current_idx = main_frames.len();
        let last = current_idx.checked_sub(1).and_then(get_main_frame);
        let main = &headers.main_frames;

        if kind == FrameKind::Intra {
            main.parse_intra(data, headers, last)
        } else {
            let last_last = current_idx.checked_sub(2).and_then(get_main_frame);
            let skipped = 0; // FIXME

            main.parse_inter(data, headers, last, last_last, skipped)
        }
    }

    pub(crate) fn get(&self, index: usize, headers: &Headers) -> Option<MainValue> {
        let unit = match index {
            0 => MainValue::Unsigned(self.iteration),
            1 => MainValue::FrameTime(self.time),
            _ => {
                let index = index - 2;
                let def = headers.main_frames.fields.get(index)?;
                let raw = self.values[index];
                match def.unit {
                    MainUnit::Amperage => {
                        debug_assert!(def.signed);
                        MainValue::Amperage(units::Amperage::new(raw, headers))
                    }
                    MainUnit::Voltage => {
                        debug_assert!(!def.signed);
                        MainValue::Voltage(units::Voltage::new(raw, headers))
                    }
                    MainUnit::Acceleration => {
                        debug_assert!(def.signed);
                        MainValue::Acceleration(units::Acceleration::new(raw, headers))
                    }
                    MainUnit::Rotation => {
                        debug_assert!(def.signed);
                        MainValue::Rotation(units::Rotation::new(raw))
                    }
                    MainUnit::Unitless => MainValue::new_unitless(raw, def.signed),
                    MainUnit::FrameTime => unreachable!(),
                }
            }
        };

        Some(unit)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MainValue {
    FrameTime(u64),
    Amperage(units::Amperage),
    Voltage(units::Voltage),
    Acceleration(units::Acceleration),
    Rotation(units::Rotation),
    Unsigned(u32),
    Signed(i32),
}

impl MainValue {
    fn new_unitless(value: u32, signed: bool) -> Self {
        if signed {
            Self::Signed(as_signed(value))
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

    pub(crate) fn has_motor_0(&self) -> bool {
        self.index_motor_0.is_some()
    }

    /// # Panics
    ///
    /// Panics if there is no motor[0] field in the frame
    pub(crate) fn get_motor_0_from(&self, frame: &[u32]) -> u32 {
        frame[self.index_motor_0.unwrap()]
    }

    pub(crate) fn validate(
        &self,
        check_predictor: impl Fn(&'data str, Predictor) -> ParseResult<()>,
        check_unit: impl Fn(&'data str, Unit) -> ParseResult<()>,
    ) -> ParseResult<()> {
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

        let mut values = read_field_values(data, &self.fields, |f| f.encoding_intra)?;

        for (i, field) in self.fields.iter().enumerate() {
            let raw = values[i];
            let signed = field.encoding_intra.is_signed();

            let last = last.map(|l| l.values[i]);

            trace_field!(pre, field = field, enc = field.encoding_intra, raw = raw);

            values[i] = field
                .predictor_intra
                .apply(headers, raw, signed, &values, last, None, 0);

            trace_field!(
                post,
                field = field,
                pred = field.predictor_intra,
                final = values[i]
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

        let mut values = read_field_values(data, &self.fields, |f| f.encoding_inter)?;

        for (i, field) in self.fields.iter().enumerate() {
            let raw = values[i];
            let signed = field.encoding_inter.is_signed();

            let last = last.map(|l| l.values[i]);
            let last_last = last_last.map(|l| l.values[i]);

            trace_field!(pre, field = field, enc = field.encoding_inter, raw = raw);

            values[i] = field.predictor_inter.apply(
                headers,
                raw,
                signed,
                &values,
                last,
                last_last,
                skipped_frames,
            );

            trace_field!(
                post,
                field = field,
                pred = field.predictor_inter,
                final = values[i]
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

    pub(crate) fn parse(self) -> ParseResult<MainFrameDef<'data>> {
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
            return Err(ParseError::MissingField(
                FrameKind::Intra,
                "loopIteration".to_owned(),
            ));
        };

        let Some(time @ MainFieldDef {
            name: "time",
            predictor_intra: Predictor::Zero,
            predictor_inter: Predictor::StraightLine,
            encoding_intra: Encoding::Variable,
            encoding_inter: Encoding::VariableSigned,
            ..
        }) = fields.next().transpose()? else {
            return Err(ParseError::MissingField(
                FrameKind::Intra,
                "time".to_owned(),
            ));
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
        "vbatLatest" => MainUnit::Voltage,
        "amperageLatest" => MainUnit::Amperage,
        "accSmooth" => MainUnit::Acceleration,
        "gyroADC" => MainUnit::Rotation,
        _ => MainUnit::Unitless,
    }
}
