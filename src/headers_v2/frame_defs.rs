use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::str::FromStr;
use core::{fmt, num};

use crate::parser::Encoding;
use crate::predictor::Predictor;
use crate::utils::{as_i8, as_u8};

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("invalid frame type: {0:?}")]
    InvalidFrameType(String),
    #[error("invalid frame property: {0:?}")]
    InvalidFrameProperty(String),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum BuildError {
    #[error("not declared")]
    NotDeclared,
    #[error("missing header: `{0}`")]
    MissingHeader(&'static str),
    #[error("invalid value in header: `{0}`")]
    InvalidValue(&'static str),
    #[error("unmatched lengths")]
    UnmatchedLengths,
}

macro_rules! impl_build {
    (
        $self:ident;
        $( $f:ident $(-> $g:ident)? : $s:expr $(, $map:expr)?; )+
        |..| $build:expr
    ) => {
        if $($self.$f.is_none())&&+ {
            return Err(BuildError::NotDeclared);
        }

        $(
            let mut $f = match $self.$f {
                Some(s) => s.split(','),
                None => return Err(BuildError::MissingHeader($s)),
            };
        )+

        let ret = {
            $(let $f = $f.by_ref();)+
            impl_build!(_zip $($f),+)
                .map(|impl_build!(_pat $($f),+)| {
                    $(
                        // Apply $map and handle error
                        $(let $f = $map($f).ok_or(BuildError::InvalidValue($s))?;)?
                        // Alias, if given
                        $(let $g = $f;)?
                    )+
                    Ok($build)
                })
                .collect::<Result<_, _>>()?
        };
        // Check that all properties had the same number of fields
        if $($f.next().is_some())||+ {
            return Err(BuildError::UnmatchedLengths);
        }
        Ok(ret)
    };
    (_zip $head:expr, $next:expr $(, $tail:expr)*) => {
        impl_build!(_zip $head.zip($next) $(, $tail)*)
    };
    (_zip $done:expr) => {
        $done
    };
    (_pat $head:pat, $next:pat $(, $tail:pat)*) => {
        impl_build!(_pat ($head, $next) $(, $tail)*)
    };
    (_pat $done:pat) => {
        $done
    };
}

#[derive(Debug)]
pub struct MainBuilder<'data> {
    name: Option<&'data str>,
    predictor_intra: Option<&'data str>,
    predictor_inter: Option<&'data str>,
    encoding_intra: Option<&'data str>,
    encoding_inter: Option<&'data str>,
    signed: Option<&'data str>,
}

impl<'data> MainBuilder<'data> {
    pub const fn new() -> Self {
        Self {
            name: None,
            predictor_intra: None,
            predictor_inter: None,
            encoding_intra: None,
            encoding_inter: None,
            signed: None,
        }
    }

    fn update(&mut self, is_intra: bool, property: FrameProperty, value: &'data str) {
        match (property, is_intra) {
            (FrameProperty::Name, _) => self.name = Some(value),
            (FrameProperty::Predictor, true) => self.predictor_intra = Some(value),
            (FrameProperty::Predictor, false) => self.predictor_inter = Some(value),
            (FrameProperty::Encoding, true) => self.encoding_intra = Some(value),
            (FrameProperty::Encoding, false) => self.encoding_inter = Some(value),
            (FrameProperty::Signed, _) => self.signed = Some(value),
        }
    }

    pub fn build(&self) -> Result<Vec<MainField<'data>>, BuildError> {
        impl_build! {
            self;
            name -> raw_name: "Field I name";
            predictor_intra: "Field I predictor", Predictor::from_num_str;
            predictor_inter: "Field P predictor", Predictor::from_num_str;
            encoding_intra: "Field I encoding", Encoding::from_num_str;
            encoding_inter: "Field P encoding", Encoding::from_num_str;
            signed: "Field X signed", parse_signed;
            |..| {
                let (name, index) = parse_field_name(raw_name);
                MainField {
                    raw_name,
                    kind: MainFieldKind::new(name, index),
                    index,
                    predictor_intra,
                    predictor_inter,
                    encoding_intra,
                    encoding_inter,
                    signed,
                }
            }
        }
    }
}

impl Default for MainBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct CommonBuilder<'data> {
    name: Option<&'data str>,
    predictor: Option<&'data str>,
    encoding: Option<&'data str>,
    signed: Option<&'data str>,
}

impl<'data> CommonBuilder<'data> {
    const fn new() -> Self {
        Self {
            name: None,
            predictor: None,
            encoding: None,
            signed: None,
        }
    }

    fn update(&mut self, property: FrameProperty, value: &'data str) {
        match property {
            FrameProperty::Name => self.name = Some(value),
            FrameProperty::Predictor => self.predictor = Some(value),
            FrameProperty::Encoding => self.encoding = Some(value),
            FrameProperty::Signed => self.signed = Some(value),
        }
    }
}

macro_rules! impl_common_frame {
    ($builder:ident, $frame:ident, $field:ident, $kind:ty, $raw_kind:literal) => {
        #[derive(Debug)]
        pub struct $builder<'data>(CommonBuilder<'data>);

        #[derive(Debug)]
        pub struct $frame<'data>(Vec<$field<'data>>);

        impl FrameDef for $frame<'_> {
            fn len(&self) -> usize {
                self.0.len()
            }

            fn encodings(&self) -> impl Iterator<Item = Encoding> {
                self.0.iter().map(|f| f.0.encoding)
            }
        }

        #[derive(Debug)]
        pub struct $field<'data>(CommonField<'data, $kind>);

        impl<'data> $builder<'data> {
            pub const fn new() -> Self {
                Self(CommonBuilder::new())
            }

            fn update(&mut self, property: FrameProperty, value: &'data str) {
                self.0.update(property, value);
            }

            // TODO: move this onto CommonField?
            pub fn build(&self) -> Result<Vec<$field<'data>>, BuildError> {
                // For some reason $self has to be an ident or other parts of it break...
                let inner = &self.0;
                impl_build! {
                    inner;
                    name -> raw_name: concat!("Field ", $raw_kind, " name");
                    predictor: concat!("Field ", $raw_kind, " predictor"), Predictor::from_num_str;
                    encoding: concat!("Field ", $raw_kind, " encoding"), Encoding::from_num_str;
                    signed: concat!("Field ", $raw_kind, " signed"), parse_signed;
                    |..| {
                        let (name, index) = parse_field_name(raw_name);
                        $field(CommonField {
                            raw_name,
                            kind: <$kind>::new(name, index),
                            index,
                            predictor,
                            encoding,
                            signed,
                        })
                    }
                }
            }
        }

        impl Default for $builder<'_> {
            fn default() -> Self {
                Self::new()
            }
        }

        impl Field for $field<'_> {
            type Kind = $kind;

            fn kind(&self) -> Self::Kind {
                self.0.kind()
            }

            fn index(&self) -> Option<u8> {
                self.0.index()
            }

            fn raw_kind(&self) -> &str {
                self.0.raw_kind()
            }

            fn raw_name(&self) -> &str {
                self.0.raw_name()
            }

            fn signed(&self) -> bool {
                self.0.signed()
            }
        }
    };
}

impl_common_frame!(SlowBuilder, SlowFrameDef, SlowField, SlowFieldKind, "S");
impl_common_frame!(GpsBuilder, GpsFrameDef, GpsField, GpsFieldKind, "G");
impl_common_frame!(
    GpsHomeBuilder,
    GpsHomeFrameDef,
    GpsHomeField,
    GpsHomeFieldKind,
    "H"
);

#[derive(Debug)]
pub struct FrameDefBuilders<'data> {
    pub main: MainBuilder<'data>,
    pub slow: SlowBuilder<'data>,
    pub gps: GpsBuilder<'data>,
    pub gps_home: GpsHomeBuilder<'data>,
}

impl<'data> FrameDefBuilders<'data> {
    pub const fn new() -> Self {
        Self {
            main: MainBuilder::new(),
            slow: SlowBuilder::new(),
            gps: GpsBuilder::new(),
            gps_home: GpsHomeBuilder::new(),
        }
    }

    pub fn update(&mut self, header: &'data str, value: &'data str) -> Result<(), ParseError> {
        let raw_header = header;

        let Some(header) = header.strip_prefix("Field ") else {
            return Ok(());
        };

        let Some((kind, property)) = header.split_once(' ') else {
            tracing::debug!(
                "skipping header that initially seemed to be a frame definition: {raw_header:?}"
            );
            return Ok(());
        };

        let kind = kind
            .parse()
            .map_err(|_| ParseError::InvalidFrameType(kind.to_owned()))?;
        let property = property
            .parse()
            .map_err(|_| ParseError::InvalidFrameProperty(property.to_owned()))?;

        match kind {
            FrameKind::Intra => self.main.update(true, property, value),
            FrameKind::Inter => self.main.update(false, property, value),
            FrameKind::Slow => self.slow.update(property, value),
            FrameKind::Gps => self.gps.update(property, value),
            FrameKind::GpsHome => self.gps_home.update(property, value),
        }

        Ok(())
    }

    pub fn build(self) -> Result<FrameDefs<'data>, BuildError> {
        fn optional<T>(r: Result<T, BuildError>) -> Result<Option<T>, BuildError> {
            match r {
                Ok(ok) => Ok(Some(ok)),
                Err(BuildError::NotDeclared) => Ok(None),
                Err(err) => Err(err),
            }
        }

        Ok(FrameDefs {
            main: MainFrameDef(self.main.build()?),
            slow: SlowFrameDef(self.slow.build()?),
            gps: optional(self.gps.build())?.map(GpsFrameDef),
            gps_home: optional(self.gps_home.build())?.map(GpsHomeFrameDef),
        })
    }
}

impl Default for FrameDefBuilders<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct FrameDefs<'data> {
    pub main: MainFrameDef<'data>,
    pub slow: SlowFrameDef<'data>,
    pub gps: Option<GpsFrameDef<'data>>,
    pub gps_home: Option<GpsHomeFrameDef<'data>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum FrameKind {
    Intra = b'I',
    Inter = b'P',
    Slow = b'S',
    Gps = b'G',
    GpsHome = b'H',
}

impl FromStr for FrameKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 || !s.is_ascii() {
            return Err(());
        }

        match s.as_bytes()[0] {
            b'I' => Ok(Self::Intra),
            b'P' => Ok(Self::Inter),
            b'S' => Ok(Self::Slow),
            b'G' => Ok(Self::Gps),
            b'H' => Ok(Self::GpsHome),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameProperty {
    Name,
    Predictor,
    Encoding,
    Signed,
}

impl FromStr for FrameProperty {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Self::Name),
            "predictor" => Ok(Self::Predictor),
            "encoding" => Ok(Self::Encoding),
            "signed" => Ok(Self::Signed),
            _ => Err(()),
        }
    }
}

pub trait Field {
    type Kind: Copy + Eq;

    fn kind(&self) -> Self::Kind;
    fn index(&self) -> Option<u8>;
    fn raw_kind(&self) -> &str;
    fn raw_name(&self) -> &str;
    fn signed(&self) -> bool;
}

pub(crate) trait FrameDef {
    fn len(&self) -> usize;
    fn encodings(&self) -> impl Iterator<Item = Encoding>;
}

impl<T: FrameDef> FrameDef for &'_ T {
    fn len(&self) -> usize {
        (*self).len()
    }

    fn encodings(&self) -> impl Iterator<Item = Encoding> {
        (*self).encodings()
    }
}

#[derive(Debug)]
pub struct MainFrameDef<'data>(Vec<MainField<'data>>);

#[derive(Debug)]
pub(crate) struct MainFrameDefRef<'f, 'data> {
    fields: &'f [MainField<'data>],
    kind: MainFrameKind,
}

#[derive(Debug, Clone, Copy)]
enum MainFrameKind {
    Intra,
    Inter,
}

impl<'data> MainFrameDef<'data> {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn as_intra<'f>(&'f self) -> MainFrameDefRef<'f, 'data> {
        MainFrameDefRef {
            fields: &self.0,
            kind: MainFrameKind::Intra,
        }
    }

    pub(crate) fn as_inter<'f>(&'f self) -> MainFrameDefRef<'f, 'data> {
        MainFrameDefRef {
            fields: &self.0,
            kind: MainFrameKind::Inter,
        }
    }
}

impl FrameDef for MainFrameDefRef<'_, '_> {
    fn len(&self) -> usize {
        self.fields.len()
    }

    fn encodings(&self) -> impl Iterator<Item = Encoding> {
        let map = match self.kind {
            MainFrameKind::Intra => |f: &MainField<'_>| f.encoding_intra,
            MainFrameKind::Inter => |f: &MainField<'_>| f.encoding_inter,
        };

        self.fields.iter().map(map)
    }
}

#[derive(Debug, Clone)]
pub struct MainField<'data> {
    raw_name: &'data str,
    kind: MainFieldKind,
    index: FieldIndex,
    predictor_intra: Predictor,
    predictor_inter: Predictor,
    encoding_intra: Encoding,
    encoding_inter: Encoding,
    signed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MainFieldKind {
    Unknown,
}

impl MainFieldKind {
    fn new(_name: &str, _index: FieldIndex) -> Self {
        Self::Unknown
    }
}

impl Field for MainField<'_> {
    type Kind = MainFieldKind;

    fn kind(&self) -> Self::Kind {
        self.kind
    }

    fn index(&self) -> Option<u8> {
        self.index.into()
    }

    fn raw_kind(&self) -> &str {
        self.raw_name
            .split_once('[')
            .map_or(self.raw_name, |(s, _)| s)
    }

    fn raw_name(&self) -> &str {
        self.raw_name
    }

    fn signed(&self) -> bool {
        self.signed
    }
}

#[derive(Debug)]
struct CommonField<'data, K> {
    raw_name: &'data str,
    kind: K,
    index: FieldIndex,
    predictor: Predictor,
    encoding: Encoding,
    signed: bool,
}

impl<K: Copy + Eq> Field for CommonField<'_, K> {
    type Kind = K;

    fn kind(&self) -> Self::Kind {
        self.kind
    }

    fn index(&self) -> Option<u8> {
        self.index.into()
    }

    fn raw_kind(&self) -> &str {
        self.raw_name
            .split_once('[')
            .map_or(self.raw_name, |(s, _)| s)
    }

    fn raw_name(&self) -> &str {
        self.raw_name
    }

    fn signed(&self) -> bool {
        self.signed
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SlowFieldKind {
    FailsafePhase,
    #[default]
    Unknown,
}

impl SlowFieldKind {
    fn new(_name: &str, _index: FieldIndex) -> Self {
        match _name {
            "failsafePhase" => Self::FailsafePhase,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GpsFieldKind {
    #[default]
    Unknown,
}

impl GpsFieldKind {
    fn new(_name: &str, _index: FieldIndex) -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GpsHomeFieldKind {
    Latittude,
    Longitude,
    #[default]
    Unknown,
}

impl GpsHomeFieldKind {
    fn new(name: &str, index: FieldIndex) -> Self {
        match (name, index.0) {
            ("GPS_home", 0) => Self::Latittude,
            ("GPS_home", 1) => Self::Longitude,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy)]
struct FieldIndex(i8);

impl fmt::Debug for FieldIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FieldIndex")
            .field(&Option::<u8>::from(*self))
            .finish()
    }
}

impl From<FieldIndex> for Option<u8> {
    fn from(FieldIndex(index): FieldIndex) -> Self {
        if index.is_negative() {
            None
        } else {
            Some(as_u8(index))
        }
    }
}

fn parse_field_name(raw: &str) -> (&str, FieldIndex) {
    let default = (raw, FieldIndex(-1));

    let Some(raw) = raw.strip_suffix(']') else {
        return default;
    };
    let Some((name, index)) = raw.split_once('[') else {
        return default;
    };

    match index.parse::<u8>() {
        Ok(index) => (name, FieldIndex(as_i8(index))),
        Err(err) if matches!(err.kind(), num::IntErrorKind::PosOverflow) => (name, FieldIndex(-1)),
        Err(_) => default,
    }
}

fn parse_signed(raw: &str) -> Option<bool> {
    match raw {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    }
}
