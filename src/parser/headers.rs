use crate::{FrameDef, FrameDefs, FrameKind, LogVersion, ParseResult};
use biterator::Biterator;
use std::collections::HashMap;
use std::io::Read;
use std::str;

fn parse_header<R: Read>(log: &mut Biterator<R>) -> (String, String) {
    assert_eq!(Some(b'H'), log.next_byte());
    log.next_byte_if_eq(b' ');

    let line = log.bytes().take_while(|b| *b != b'\n').collect::<Vec<_>>();

    let line = str::from_utf8(&line).unwrap();
    let (name, value) = line.split_once(':').unwrap();

    (name.to_owned(), value.to_owned())
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

#[derive(Debug)]
pub struct Headers {
    pub(crate) version: LogVersion,
    pub(crate) frames: FrameDefs,
    pub(crate) unknown: HashMap<String, String>,
}

impl Headers {
    pub fn parse<R: Read>(log: &mut Biterator<R>) -> ParseResult<Self> {
        let (name, product) = parse_header(log);
        assert_eq!(name, "Product", "`Product` header must be first");
        let (name, version) = parse_header(log);
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

        while log.peek_byte() == Some(b'H') {
            let (name, value) = parse_header(log);

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
