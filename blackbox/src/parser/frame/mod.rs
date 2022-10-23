mod gps;
mod gps_home;
mod main;
mod slow;

use alloc::vec::Vec;
use core::iter::Peekable;

pub use self::gps::*;
pub use self::gps_home::*;
pub use self::main::*;
pub use self::slow::*;
use super::{Encoding, ParseError, ParseResult, Predictor, Reader};

pub trait FieldDef {
    fn name(&self) -> &str;
    fn predictor(&self) -> Predictor;
    fn encoding(&self) -> Encoding;
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
    Gps,
    GpsHome,
    Intra,
    Inter,
    Slow,
}

impl FrameKind {
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

impl From<FrameKind> for char {
    fn from(kind: FrameKind) -> Self {
        match kind {
            FrameKind::Gps => 'G',
            FrameKind::GpsHome => 'H',
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
    Signed,
}

impl FrameProperty {
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

fn missing_header_error(kind: FrameKind, property: &'static str) -> ParseError {
    tracing::error!("missing header `Field {} {property}`", char::from(kind));
    ParseError::Corrupted
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
    let s = s.ok_or_else(|| missing_header_error(kind, property))?;

    let iter = s.split(',').map(|s| {
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

fn parse_signs(
    kind: FrameKind,
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
