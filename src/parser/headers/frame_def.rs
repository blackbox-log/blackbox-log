use std::borrow::ToOwned;

use crate::parser::{Encoding, ParseError, ParseResult, Predictor};

#[derive(Debug)]
pub struct FrameDefs {
    pub(in crate::parser) intra: Vec<FieldDef>,
    pub(in crate::parser) inter: Vec<FieldDef>,
    pub(in crate::parser) slow: Vec<FieldDef>,
}

#[derive(Debug)]
pub struct FieldDef {
    pub(in crate::parser) name: String,
    pub(in crate::parser) signed: bool,
    pub(in crate::parser) predictor: Predictor,
    pub(in crate::parser) encoding: Encoding,
}

#[derive(Debug, Default)]
pub(super) struct Builders {
    pub intra: Builder,
    pub inter: Builder,
    pub slow: Builder,
}

impl Builders {
    pub(super) fn parse(mut self) -> ParseResult<FrameDefs> {
        self.inter.names = self.intra.names.clone();
        self.inter.signs = self.intra.signs.clone();

        let intra = self.intra.parse()?;
        let inter = self.inter.parse()?;
        let slow = self.slow.parse()?;

        Ok(FrameDefs { intra, inter, slow })
    }
}

#[derive(Debug, Default)]
pub(super) struct Builder {
    pub names: Option<String>,
    pub signs: Option<String>,
    pub predictors: Option<String>,
    pub encodings: Option<String>,
}

impl Builder {
    fn parse(self) -> ParseResult<Vec<FieldDef>> {
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

        Ok(fields)
    }
}
