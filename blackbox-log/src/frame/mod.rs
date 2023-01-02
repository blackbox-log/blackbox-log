//! Types for blackbox log data frames.

#[macro_use]
mod trace_field;

pub mod gps;
mod gps_home;
pub mod main;
pub mod slow;

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::vec::Vec;
use core::fmt;
use core::iter::Peekable;

pub(crate) use self::gps::*;
pub use self::gps::{GpsFrame, GpsUnit, GpsValue};
pub(crate) use self::gps_home::*;
pub(crate) use self::main::*;
pub use self::main::{MainFrame, MainUnit, MainValue};
pub(crate) use self::slow::*;
pub use self::slow::{SlowFrame, SlowUnit, SlowValue};
use crate::parser::{Encoding, InternalResult};
use crate::predictor::{Predictor, PredictorContext};
use crate::units::prelude::*;
use crate::{units, HeadersParseError, HeadersParseResult, Reader};

pub trait Frame {
    type Value: Into<Value>;

    fn get(&self, index: usize) -> Option<Self::Value>;

    fn iter(&self) -> FrameIter<'_, Self>
    where
        Self: Sized,
    {
        FrameIter {
            frame: self,
            next: 0,
        }
    }
}

#[derive(Debug)]
pub struct FrameIter<'f, F> {
    frame: &'f F,
    next: usize,
}

impl<F: Frame> Iterator for FrameIter<'_, F> {
    type Item = F::Value;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.frame.get(self.next)?;
        self.next += 1;
        Some(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
pub enum FrameKind {
    Event,
    Data(DataFrameKind),
}

impl FrameKind {
    pub(crate) const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'E' => Some(Self::Event),
            _ => {
                if let Some(kind) = DataFrameKind::from_byte(byte) {
                    Some(Self::Data(kind))
                } else {
                    None
                }
            }
        }
    }
}

impl From<FrameKind> for char {
    fn from(kind: FrameKind) -> Self {
        match kind {
            FrameKind::Event => 'E',
            FrameKind::Data(kind) => kind.into(),
        }
    }
}

impl From<FrameKind> for u8 {
    fn from(kind: FrameKind) -> Self {
        match kind {
            FrameKind::Event => b'E',
            FrameKind::Data(kind) => kind.into(),
        }
    }
}

impl fmt::Display for FrameKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Event => f.write_str("event"),
            Self::Data(kind) => kind.fmt(f),
        }
    }
}

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[repr(u8)]
    pub enum DataFrameKind {
        Intra = b'I',
        Inter = b'P',
        Gps = b'G',
        GpsHome = b'H',
        Slow = b'S',
    }
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

impl fmt::Display for DataFrameKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self {
            Self::Intra => "intra",
            Self::Inter => "inter",
            Self::Gps => "GPS",
            Self::GpsHome => "GPS home",
            Self::Slow => "slow",
        };

        f.write_str(kind)
    }
}

trait FieldDef<'data> {
    fn name(&self) -> &'data str;
    fn predictor(&self) -> Predictor;
    fn encoding(&self) -> Encoding;
    fn signed(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Unit {
    FrameTime,
    Amperage,
    Voltage,
    Acceleration,
    Rotation,
    FlightMode,
    State,
    FailsafePhase,
    GpsCoordinate,
    Altitude,
    Velocity,
    GpsHeading,
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

impl From<GpsUnit> for Unit {
    fn from(unit: GpsUnit) -> Self {
        match unit {
            GpsUnit::FrameTime => Self::FrameTime,
            GpsUnit::Coordinate => Self::GpsCoordinate,
            GpsUnit::Altitude => Self::Altitude,
            GpsUnit::Velocity => Self::Velocity,
            GpsUnit::Heading => Self::GpsHeading,
            GpsUnit::Unitless => Self::Unitless,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    FrameTime(Time),
    Amperage(ElectricCurrent),
    Voltage(ElectricPotential),
    Acceleration(Acceleration),
    Rotation(AngularVelocity),
    FlightMode(units::FlightModeSet),
    State(units::StateSet),
    FailsafePhase(units::FailsafePhase),
    Boolean(bool),
    GpsCoordinate(f64),
    Altitude(Length),
    Velocity(Velocity),
    GpsHeading(f64),
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

impl From<GpsValue> for Value {
    fn from(value: GpsValue) -> Self {
        match value {
            GpsValue::FrameTime(t) => Self::FrameTime(t),
            GpsValue::Coordinate(c) => Self::GpsCoordinate(c),
            GpsValue::Altitude(a) => Self::Altitude(a),
            GpsValue::Velocity(v) => Self::Velocity(v),
            GpsValue::Heading(h) => Self::GpsHeading(h),
            GpsValue::Unsigned(x) => Self::Unsigned(x),
            GpsValue::Signed(x) => Self::Signed(x),
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

fn missing_header_error(kind: DataFrameKind, property: &'static str) -> HeadersParseError {
    tracing::error!("missing header `Field {} {property}`", char::from(kind));
    HeadersParseError::MissingHeader
}

fn parse_names(
    kind: DataFrameKind,
    names: Option<&str>,
) -> HeadersParseResult<impl Iterator<Item = &'_ str>> {
    let names = names.ok_or_else(|| missing_header_error(kind, "name"))?;
    Ok(names.split(','))
}

fn parse_enum_list<'a, T>(
    kind: DataFrameKind,
    property: &'static str,
    s: Option<&'a str>,
    parse: impl Fn(&str) -> Option<T> + 'a,
) -> HeadersParseResult<impl Iterator<Item = HeadersParseResult<T>> + 'a> {
    let s = s.ok_or_else(|| missing_header_error(kind, property))?;
    Ok(s.split(',').map(move |s| {
        parse(s).ok_or_else(|| HeadersParseError::InvalidHeader {
            header: format!("Field {} {property}", char::from(kind)),
            value: s.to_owned(),
        })
    }))
}

#[inline]
fn parse_predictors(
    kind: DataFrameKind,
    predictors: Option<&'_ str>,
) -> HeadersParseResult<impl Iterator<Item = HeadersParseResult<Predictor>> + '_> {
    parse_enum_list(kind, "predictor", predictors, Predictor::from_num_str)
}

#[inline]
fn parse_encodings(
    kind: DataFrameKind,
    encodings: Option<&'_ str>,
) -> HeadersParseResult<impl Iterator<Item = HeadersParseResult<Encoding>> + '_> {
    parse_enum_list(kind, "encoding", encodings, Encoding::from_num_str)
}

fn parse_signs(
    kind: DataFrameKind,
    names: Option<&str>,
) -> HeadersParseResult<impl Iterator<Item = bool> + '_> {
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
) -> InternalResult<Vec<u32>> {
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

fn parse_impl<'data, F: FieldDef<'data>>(
    mut ctx: PredictorContext<'_, 'data>,
    raw: &[u32],
    fields: impl IntoIterator<Item = F>,
    update_ctx: impl Fn(&mut PredictorContext<'_, 'data>, usize),
) -> Vec<u32> {
    let mut values = Vec::with_capacity(raw.len());

    for (i, field) in fields.into_iter().enumerate() {
        let encoding = field.encoding();
        let predictor = field.predictor();

        let raw = raw[i];
        let signed = encoding.is_signed();

        update_ctx(&mut ctx, i);

        trace_field!(pre, field = field, enc = encoding, raw = raw);

        let value = predictor.apply(raw, signed, Some(&values), &ctx);
        values.push(value);

        trace_field!(
            post,
            field = field,
            pred = predictor,
            final = value
        );
    }

    values
}
