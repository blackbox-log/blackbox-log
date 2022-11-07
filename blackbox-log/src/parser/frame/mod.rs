mod gps;
mod gps_home;
mod main;
mod slow;

use alloc::vec::Vec;
use core::iter::Peekable;

pub(crate) use self::gps::*;
pub(crate) use self::gps_home::*;
pub(crate) use self::main::*;
pub(crate) use self::slow::*;
use super::{Encoding, ParseError, ParseResult, Predictor, Reader};
use crate::units;

pub trait FieldDef {
    fn name(&self) -> &str;
    fn predictor(&self) -> Predictor;
    fn encoding(&self) -> Encoding;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Unit {
    FrameTime,
    Amperage,
    Voltage,
    Acceleration,
    Rotation,
    FlightMode,
    State,
    FailsafePhase,
    Boolean,
    Unitless,
}

impl From<MainUnit> for Unit {
    fn from(unit: MainUnit) -> Self {
        match unit {
            MainUnit::FrameTime => Self::FrameTime,
            MainUnit::Amperage => Self::Amperage,
            MainUnit::Voltage => Self::Voltage,
            MainUnit::Acceleration => Self::Acceleration,
            MainUnit::Rotation => Self::Rotation,
            MainUnit::Unitless => Self::Unitless,
        }
    }
}

impl From<SlowUnit> for Unit {
    fn from(unit: SlowUnit) -> Self {
        match unit {
            SlowUnit::FlightMode => Self::FlightMode,
            SlowUnit::State => Self::State,
            SlowUnit::FailsafePhase => Self::FailsafePhase,
            SlowUnit::Boolean => Self::Boolean,
            SlowUnit::Unitless => Self::Unitless,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Value {
    FrameTime(u64),
    Amperage(units::Amperage),
    Voltage(units::Voltage),
    Acceleration(units::Acceleration),
    Rotation(units::Rotation),
    FlightMode(units::FlightModeSet),
    State(units::StateSet),
    FailsafePhase(units::FailsafePhaseSet),
    Boolean(bool),
    Unsigned(u32),
    Signed(i32),
}

impl From<MainValue> for Value {
    fn from(value: MainValue) -> Self {
        match value {
            MainValue::FrameTime(t) => Self::FrameTime(t),
            MainValue::Amperage(a) => Self::Amperage(a),
            MainValue::Voltage(v) => Self::Voltage(v),
            MainValue::Acceleration(a) => Self::Acceleration(a),
            MainValue::Rotation(r) => Self::Rotation(r),
            MainValue::Unsigned(x) => Self::Unsigned(x),
            MainValue::Signed(x) => Self::Signed(x),
        }
    }
}

impl From<SlowValue> for Value {
    fn from(value: SlowValue) -> Self {
        match value {
            SlowValue::FlightMode(m) => Self::FlightMode(m),
            SlowValue::State(s) => Self::State(s),
            SlowValue::FailsafePhase(p) => Self::FailsafePhase(p),
            SlowValue::Boolean(b) => Self::Boolean(b),
            SlowValue::Unsigned(x) => Self::Unsigned(x),
            SlowValue::Signed(x) => Self::Signed(x),
        }
    }
}

pub(crate) fn is_frame_def_header(header: &str) -> bool {
    parse_frame_def_header(header).is_some()
}

pub(crate) fn parse_frame_def_header(header: &str) -> Option<(DataFrameKind, DataFrameProperty)> {
    let header = header.strip_prefix("Field ")?;
    let (kind, property) = header.split_once(' ')?;

    Some((
        DataFrameKind::from_letter(kind)?,
        DataFrameProperty::from_name(property)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataFrameKind {
    Gps,
    GpsHome,
    Intra,
    Inter,
    Slow,
}

impl DataFrameKind {
    pub(crate) fn from_letter(s: &str) -> Option<Self> {
        match s {
            "G" => Some(Self::Gps),
            "H" => Some(Self::GpsHome),
            "I" => Some(Self::Intra),
            "P" => Some(Self::Inter),
            "S" => Some(Self::Slow),
            _ => None,
        }
    }
}

impl From<DataFrameKind> for char {
    fn from(kind: DataFrameKind) -> Self {
        match kind {
            DataFrameKind::Gps => 'G',
            DataFrameKind::GpsHome => 'H',
            DataFrameKind::Intra => 'I',
            DataFrameKind::Inter => 'P',
            DataFrameKind::Slow => 'S',
        }
    }
}

// TODO: width?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataFrameProperty {
    Name,
    Predictor,
    Encoding,
    Signed,
}

impl DataFrameProperty {
    pub(crate) fn from_name(s: &str) -> Option<Self> {
        match s {
            "name" => Some(Self::Name),
            "predictor" => Some(Self::Predictor),
            "encoding" => Some(Self::Encoding),
            "signed" => Some(Self::Signed),
            _ => None,
        }
    }
}

fn missing_header_error(kind: DataFrameKind, property: &'static str) -> ParseError {
    tracing::error!("missing header `Field {} {property}`", char::from(kind));
    ParseError::Corrupted
}

fn parse_names(
    kind: DataFrameKind,
    names: Option<&str>,
) -> ParseResult<impl Iterator<Item = &'_ str>> {
    let names = names.ok_or_else(|| missing_header_error(kind, "name"))?;
    Ok(names.split(','))
}

fn parse_enum_list<'a, T>(
    kind: DataFrameKind,
    property: &'static str,
    s: Option<&'a str>,
    parse: impl Fn(&str) -> Option<T> + 'a,
) -> ParseResult<impl Iterator<Item = ParseResult<T>> + 'a> {
    let s = s.ok_or_else(|| missing_header_error(kind, property))?;
    Ok(s.split(',')
        .map(move |s| parse(s).ok_or(ParseError::Corrupted)))
}

#[inline]
fn parse_predictors(
    kind: DataFrameKind,
    predictors: Option<&'_ str>,
) -> ParseResult<impl Iterator<Item = ParseResult<Predictor>> + '_> {
    parse_enum_list(kind, "predictor", predictors, Predictor::from_num_str)
}

#[inline]
fn parse_encodings(
    kind: DataFrameKind,
    encodings: Option<&'_ str>,
) -> ParseResult<impl Iterator<Item = ParseResult<Encoding>> + '_> {
    parse_enum_list(kind, "encoding", encodings, Encoding::from_num_str)
}

fn parse_signs(
    kind: DataFrameKind,
    names: Option<&str>,
) -> ParseResult<impl Iterator<Item = bool> + '_> {
    let names = names.ok_or_else(|| missing_header_error(kind, "signed"))?;
    Ok(names.split(',').map(|s| s.trim() != "0"))
}

fn count_fields_with_same_encoding(
    fields: &mut Peekable<impl Iterator<Item = Encoding>>,
    max: usize,
    encoding: Encoding,
) -> usize {
    let mut extra = 0;
    while extra < max && fields.next_if_eq(&encoding).is_some() {
        extra += 1;
    }
    extra
}

fn read_field_values<T>(
    data: &mut Reader,
    fields: &[T],
    get_encoding: impl Fn(&T) -> Encoding,
) -> ParseResult<Vec<u32>> {
    let mut encodings = fields.iter().map(get_encoding).peekable();
    let mut values = Vec::with_capacity(encodings.len());

    while let Some(encoding) = encodings.next() {
        let extra = encoding.max_chunk_size() - 1;
        let extra = count_fields_with_same_encoding(&mut encodings, extra, encoding);

        encoding.decode_into(data, extra, &mut values)?;
    }

    debug_assert_eq!(values.len(), fields.len());

    Ok(values)
}
