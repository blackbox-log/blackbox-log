use crate::{FrameDef, FrameDefs, FrameKind, LogVersion, ParseError, ParseResult, Reader};
use bitter::BitReader;
use std::collections::HashMap;
use std::str;

// Reason: unfinished
#[allow(dead_code)]
#[derive(Debug)]
pub struct Headers {
    pub(crate) version: LogVersion,
    pub(crate) frames: FrameDefs,
    pub(crate) unknown: HashMap<String, String>,
}

impl Headers {
    pub fn parse(data: &mut Reader) -> ParseResult<Self> {
        read_h(data)?;
        let (name, _product) = parse_header(data)?;
        assert_eq!(name, "Product", "`Product` header must be first");

        read_h(data)?;
        let (name, version) = parse_header(data)?;
        assert_eq!(name, "Data version", "`Data version` header must be second");
        let version = version.parse().unwrap();

        let mut unknown = HashMap::new();

        let mut intraframe = FrameDef::builder(FrameKind::Intra);
        let mut interframe = FrameDef::builder(FrameKind::Inter);
        let mut slow = FrameDef::builder(FrameKind::Slow);

        let mut update_field_def = |name: &str, value| {
            // Skip `Field`
            let mut name = name.split(' ').skip(1);

            let frame = match name.next().unwrap() {
                "I" => &mut intraframe,
                "P" => &mut interframe,
                "S" => &mut slow,
                _ => unreachable!(),
            };

            match name.next().unwrap() {
                "name" => frame.names(value),
                "signed" => frame.signed(value),
                "width" => frame.widths(value),
                "predictor" => frame.predictors(value),
                "encoding" => frame.encodings(value),
                _ => unreachable!(),
            };
        };

        loop {
            // Need at least "H\n"
            if !data.has_bits_remaining(16) {
                return Err(ParseError::unexpected_eof());
            }
            assert!(data.refill_lookahead() >= 8);

            if data.peek(8) != b'H'.into() {
                break;
            }

            let (name, value) = parse_header(data)?;

            if is_field_def(&name) {
                update_field_def(&name, value);
            } else {
                unknown.insert(name, value);
            }
        }

        interframe.names = intraframe.names.clone();
        interframe.signs = intraframe.signs.clone();

        let intraframe = intraframe.parse();
        let interframe = interframe.parse();
        let slow = slow.parse();

        let frames = FrameDefs {
            intraframe,
            interframe,
            slow,
        };

        Ok(Self {
            version,
            frames,
            unknown,
        })
    }
}

fn read_h(data: &mut Reader) -> Result<(), ParseError> {
    match data.read_u8() {
        Some(b'H') => Ok(()),
        Some(_) => Err(ParseError::Corrupted),
        None => Err(ParseError::unexpected_eof()),
    }
}

fn parse_header(log: &mut Reader) -> ParseResult<(String, String)> {
    let mut line = Vec::new();
    while let Some(byte) = log.read_u8() {
        if byte == b'\n' {
            break;
        }

        line.push(byte);
    }

    let line = line.strip_prefix(&[b' ']).unwrap_or(&line);
    let line = str::from_utf8(line).unwrap();
    let (name, value) = line.split_once(':').unwrap();

    Ok((name.to_owned(), value.to_owned()))
}

fn is_field_def(name: &str) -> bool {
    let mut name = name.split(' ');

    name.next() == Some("Field")
        && matches!(name.next(), Some("I" | "P" | "S"))
        && matches!(
            name.next(),
            Some("name" | "signed" | "width" | "predictor" | "encoding")
        )
        && name.next() == None
}
