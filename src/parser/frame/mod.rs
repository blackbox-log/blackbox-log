mod main;
mod slow;

use std::iter::Peekable;

pub use main::*;
pub use slow::*;

use super::{Encoding, ParseError, ParseResult, Predictor};

pub trait FieldDef {
    fn name(&self) -> &str;
    fn predictor(&self) -> Predictor;
    fn encoding(&self) -> Encoding;
}

pub trait Frame {
    fn values(&self) -> &[i64];
}

pub(crate) fn is_frame_def_header(header: &str) -> bool {
    parse_frame_def_header(header).is_some()
}

pub(crate) fn parse_frame_def_header(header: &str) -> Option<(FrameKind, FrameProperty)> {
    let header = header.strip_prefix("Field ")?;
    let (kind, property) = header.split_once(' ')?;

    Some((
        FrameKind::from_letter(kind)?,
        FrameProperty::from_name(property)?,
    ))
}

// TODO: gps & gps home
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FrameKind {
    Intra,
    Inter,
    Slow,
}

impl FrameKind {
    pub(crate) fn from_letter(s: &str) -> Option<Self> {
        match s {
            "I" => Some(Self::Intra),
            "P" => Some(Self::Inter),
            "S" => Some(Self::Slow),
            _ => None,
        }
    }
}

impl From<FrameKind> for char {
    fn from(kind: FrameKind) -> Self {
        match kind {
            FrameKind::Intra => 'I',
            FrameKind::Inter => 'P',
            FrameKind::Slow => 'S',
        }
    }
}

// TODO: signed & width?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FrameProperty {
    Name,
    Predictor,
    Encoding,
}

impl FrameProperty {
    pub(crate) fn from_name(s: &str) -> Option<Self> {
        match s {
            "name" => Some(Self::Name),
            "predictor" => Some(Self::Predictor),
            "encoding" => Some(Self::Encoding),
            _ => None,
        }
    }
}

fn parse_list<'a>(
    kind: FrameKind,
    property: &'static str,
    s: Option<&'a str>,
) -> ParseResult<std::str::Split<'a, char>> {
    let s = s.ok_or_else(|| {
        ParseError::MissingHeader(format!("Field {} {property}", char::from(kind)))
    })?;
    Ok(s.split(','))
}

fn missing_header_error(kind: FrameKind, property: &'static str) -> ParseError {
    ParseError::MissingHeader(format!("Field {} {property}", char::from(kind)))
}

fn parse_names(kind: FrameKind, names: Option<&str>) -> ParseResult<impl Iterator<Item = &'_ str>> {
    let names = names.ok_or_else(|| missing_header_error(kind, "name"))?;
    Ok(names.split(','))
}

fn parse_enum_list<'a, T>(
    kind: FrameKind,
    property: &'static str,
    s: Option<&'a str>,
) -> ParseResult<impl Iterator<Item = ParseResult<T>> + 'a>
where
    T: TryFrom<u8>,
{
    let s = parse_list(kind, property, s)?;
    let iter = s.map(|s| {
        s.parse()
            .map_err(|_| ())
            .and_then(|x: u8| x.try_into().map_err(|_| ()))
            .map_err(|()| ParseError::Corrupted)
    });

    Ok(iter)
}

#[inline]
fn parse_predictors(
    kind: FrameKind,
    predictors: Option<&'_ str>,
) -> ParseResult<impl Iterator<Item = ParseResult<Predictor>> + '_> {
    parse_enum_list(kind, "predictor", predictors)
}

#[inline]
fn parse_encodings(
    kind: FrameKind,
    encodings: Option<&'_ str>,
) -> ParseResult<impl Iterator<Item = ParseResult<Encoding>> + '_> {
    parse_enum_list(kind, "encoding", encodings)
}

fn count_fields_with_same_encoding<F>(
    fields: &mut Peekable<impl Iterator<Item = F>>,
    max: usize,
    filter: impl Fn(&F) -> bool,
) -> usize {
    let mut extra = 0;
    while extra < max && fields.next_if(&filter).is_some() {
        extra += 1;
    }
    extra
}
