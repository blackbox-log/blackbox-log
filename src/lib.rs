#![warn(unsafe_code)]

pub mod betaflight;
pub mod encoding;
mod parser;

use bitter::BigEndianReader;
use bitter::BitReader;
use encoding::Encoding;
use num_enum::TryFromPrimitive;
use parser::{Data, FrameKind, Headers};
use std::fmt;
use std::io;
use std::iter;
use std::str;
use std::str::FromStr;

#[derive(Debug)]
pub enum ParseError {
    Corrupted,
    Io(io::Error),
}

impl ParseError {
    pub(crate) fn unexpected_eof() -> Self {
        Self::Io(io::Error::from(io::ErrorKind::UnexpectedEof))
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Corrupted => write!(f, "corrupted log file"),
            Self::Io(ref io) => write!(f, "IO error: {}", io),
        }
    }
}

impl std::error::Error for ParseError {}

impl<T> From<T> for ParseError
where
    T: Into<io::Error>,
{
    fn from(io: T) -> Self {
        Self::Io(io.into())
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogVersion {
    V1,
    V2,
}

impl FromStr for LogVersion {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "1" | "v1" => Ok(Self::V1),
            "2" | "v2" => Ok(Self::V2),
            _ => Err(()),
        }
    }
}

// Reason: unfinished
#[allow(dead_code)]
#[derive(Debug)]
struct FieldDef {
    name: String,
    signed: bool,
    width: usize,
    predictor: Predictor,
    encoding: Encoding,
}

#[derive(Debug)]
struct FrameDef {
    kind: FrameKind,
    fields: Vec<FieldDef>,
}

#[derive(Debug)]
struct FrameDefBuilder {
    kind: FrameKind,
    names: Option<String>,
    signs: Option<String>,
    widths: Option<String>,
    predictors: Option<String>,
    encodings: Option<String>,
}

impl FrameDef {
    fn builder(kind: FrameKind) -> FrameDefBuilder {
        FrameDefBuilder::new(kind)
    }
}

impl FrameDefBuilder {
    const fn new(kind: FrameKind) -> Self {
        Self {
            kind,
            names: None,
            signs: None,
            widths: None,
            predictors: None,
            encodings: None,
        }
    }

    fn names(&mut self, names: String) -> &mut Self {
        self.names = Some(names);
        self
    }

    fn signed(&mut self, signs: String) -> &mut Self {
        self.signs = Some(signs);
        self
    }

    fn widths(&mut self, widths: String) -> &mut Self {
        self.widths = Some(widths);
        self
    }

    fn predictors(&mut self, predictors: String) -> &mut Self {
        self.predictors = Some(predictors);
        self
    }

    fn encodings(&mut self, encodings: String) -> &mut Self {
        self.encodings = Some(encodings);
        self
    }

    fn parse(self) -> FrameDef {
        const DEFAULT_WIDTH: usize = 4;

        let names = self.names.unwrap();
        let signs = self.signs.unwrap();
        let predictors = self.predictors.unwrap();
        let encodings = self.encodings.unwrap();

        let names = names.split(',').map(ToString::to_string);
        let signs = signs.split(',').map(|sign| sign != "0");
        let widths = self
            .widths
            .iter()
            .flat_map(|widths| widths.split(',').map(str::parse))
            .chain(iter::repeat(Ok(DEFAULT_WIDTH)));
        let predictors = predictors
            .split(',')
            .map(|s| s.parse().unwrap())
            .map(|x: u8| Predictor::try_from(x).unwrap());
        let encodings = encodings
            .split(',')
            .map(|encoding| encoding.parse().unwrap())
            .map(u8::try_into);

        // Ensure that all have the same length, except for widths
        let names_len = names.clone().count();
        let all_equal_len = [
            signs.clone().count(),
            predictors.clone().count(),
            encodings.clone().count(),
        ]
        .iter()
        .all(|len| *len == names_len);
        assert!(all_equal_len);

        let fields = names
            .zip(signs)
            .zip(widths)
            .zip(predictors)
            .zip(encodings)
            .map(
                |((((name, signed), width), predictor), encoding)| FieldDef {
                    name,
                    signed,
                    width: width.unwrap_or(DEFAULT_WIDTH),
                    predictor,
                    encoding: encoding.unwrap(),
                },
            )
            .collect();

        FrameDef {
            kind: self.kind,
            fields,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
enum Predictor {
    Zero = 0,
    Previous,
    StraightLine,
    Average2,
    MinThrottle,
    Motor0,
    Increment,
    HomeLat, // TODO: check that lat = 0, lon = 1
    FifteenHundred,
    VBatRef,
    LastMainFrameTime,
    MinMotor,
    // HomeLon = 256,
}

impl Predictor {
    fn apply(self, value: i64 /*, current: i64, previous: i64, previous2: i64 */) -> i64 {
        let diff = match self {
            Self::Zero => 0,
            // Self::Previous => previous,
            // Self::StraightLine => (2 * previous) - previous2,
            // Self::Average2 => (previous + previous2) / 2,
            // Self::MinThrottle => todo!(),
            // Self::Motor0 => todo!(),
            // Self::Increment => todo!(),
            // Self::HomeLat => todo!(), // TODO: check that lat = 0, lon = 1
            Self::FifteenHundred => 1500,
            // Self::VBatRef => todo!(),
            // Self::LastMainFrameTime => todo!(),
            // Self::MinMotor => todo!(),
            // Self::HomeLon => todo!(),
            predictor => {
                tracing::warn!("found unimplemented predictor: {predictor:?}");
                0
            }
        };

        value + diff
    }
}

#[derive(Debug)]
struct FrameDefs {
    intraframe: FrameDef,
    interframe: FrameDef,
    slow: FrameDef,
}

pub(crate) type Reader<'a> = BigEndianReader<'a>;

// Reason: unfinished
#[allow(dead_code)]
#[derive(Debug)]
pub struct Log {
    headers: Headers,
    data: Data,
}

impl Log {
    pub fn new(data: &[u8]) -> ParseResult<Self> {
        let mut log = Reader::new(data);

        let headers = Headers::parse(&mut log)?;
        let data = Data::parse(&mut log, &headers)?;

        Ok(Self { headers, data })
    }
}

pub(crate) fn byte_align(data: &mut Reader) {
    while !data.byte_aligned() {
        data.consume(1);
    }
}
