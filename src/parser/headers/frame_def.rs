use std::borrow::ToOwned;
use std::ops::Index;

use crate::parser::{DataFrameKind, Encoding, ParseError, ParseResult, Predictor};

#[derive(Debug)]
pub struct FrameDefs {
    intra: FrameDef,
    inter: FrameDef,
    slow: FrameDef,
}

impl FrameDefs {
    pub(super) const fn builder() -> FrameDefsBuilder {
        FrameDefsBuilder {
            intra: FrameDef::builder(DataFrameKind::Intra),
            inter: FrameDef::builder(DataFrameKind::Inter),
            slow: FrameDef::builder(DataFrameKind::Slow),
        }
    }

    pub fn intra(&self) -> &FrameDef {
        &self.intra
    }

    pub fn inter(&self) -> &FrameDef {
        &self.inter
    }

    pub fn slow(&self) -> &FrameDef {
        &self.slow
    }
}

#[derive(Debug)]
pub struct FrameDef {
    kind: DataFrameKind,
    fields: Vec<FieldDef>,
}

impl FrameDef {
    pub(super) const fn builder(kind: DataFrameKind) -> Builder {
        Builder {
            kind,
            names: None,
            signs: None,
            predictors: None,
            encodings: None,
        }
    }

    pub(crate) const fn kind(&self) -> DataFrameKind {
        self.kind
    }

    pub fn values(&self) -> &[FieldDef] {
        self.fields.as_ref()
    }

    pub fn iter(&self) -> impl Iterator<Item = &FieldDef> {
        self.fields.iter()
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }
}

impl<T> Index<T> for FrameDef
where
    Vec<FieldDef>: Index<T>,
{
    type Output = <Vec<FieldDef> as Index<T>>::Output;

    fn index(&self, index: T) -> &Self::Output {
        &self.fields[index]
    }
}

#[derive(Debug)]
pub struct FieldDef {
    name: String,
    signed: bool,
    predictor: Predictor,
    encoding: Encoding,
}

impl FieldDef {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub const fn signed(&self) -> bool {
        self.signed
    }

    pub const fn predictor(&self) -> Predictor {
        self.predictor
    }

    pub const fn encoding(&self) -> Encoding {
        self.encoding
    }
}

#[derive(Debug)]
pub(super) struct FrameDefsBuilder {
    pub intra: Builder,
    pub inter: Builder,
    pub slow: Builder,
}

impl FrameDefsBuilder {
    pub(super) fn parse(mut self) -> ParseResult<FrameDefs> {
        self.inter.names = self.intra.names.clone();
        self.inter.signs = self.intra.signs.clone();

        let intra = self.intra.parse()?;
        let inter = self.inter.parse()?;
        let slow = self.slow.parse()?;

        Ok(FrameDefs { intra, inter, slow })
    }
}

#[derive(Debug)]
pub(super) struct Builder {
    kind: DataFrameKind,
    pub names: Option<String>,
    pub signs: Option<String>,
    pub predictors: Option<String>,
    pub encodings: Option<String>,
}

impl Builder {
    fn parse(self) -> ParseResult<FrameDef> {
        // FIXME: give these errors their own variant
        let names = self.names.ok_or(ParseError::Corrupted)?;
        let signs = self.signs.ok_or(ParseError::Corrupted)?;
        let predictors = self.predictors.ok_or(ParseError::Corrupted)?;
        let encodings = self.encodings.ok_or(ParseError::Corrupted)?;

        let names = names.split(',').map(ToOwned::to_owned).collect::<Vec<_>>();
        let signs = signs.split(',').map(|sign| sign != "0").collect::<Vec<_>>();

        // FIXME: return errors
        let predictors = predictors
            .split(',')
            .map(|s| s.parse::<u8>().unwrap())
            .map(|x| Predictor::try_from(x).unwrap())
            .collect::<Vec<_>>();
        let encodings = encodings
            .split(',')
            .map(|encoding| encoding.parse::<u8>().unwrap())
            .map(|x| x.try_into().unwrap())
            .collect::<Vec<_>>();

        let names_len = names.len();
        let all_equal_len = [signs.len(), predictors.len(), encodings.len()]
            .iter()
            .all(|len| *len == names_len);
        assert!(all_equal_len); // FIXME: return an error

        let fields = names
            .into_iter()
            .enumerate()
            .map(|(i, name)| FieldDef {
                name,
                signed: signs[i],
                predictor: predictors[i],
                encoding: encodings[i],
            })
            .collect();

        Ok(FrameDef {
            kind: self.kind,
            fields,
        })
    }
}
