use core::str;

use crate::Reader;

#[derive(Debug, Clone)]
pub enum Error {
    InvalidSyntax,
    UnexpectedEof,
    Utf8Error(str::Utf8Error),
}

#[derive(Debug, Clone)]
pub struct Headers<'data> {
    data: Reader<'data>,
}

impl<'data> Headers<'data> {
    pub(crate) const fn new(data: Reader<'data>) -> Self {
        Self { data }
    }

    pub(crate) const fn into_reader(self) -> Reader<'data> {
        self.data
    }
}

impl<'data> Iterator for Headers<'data> {
    type Item = Result<(&'data str, &'data str), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.peek() != Some(b'H') {
            // Not at the start of a header line, assume this is EOF or the body, so we're
            // done
            return None;
        }

        // Cannot be empty after `.peek() == Some(_)` above
        let line = self.data.read_line().unwrap();
        Some(parse_line(line))
    }
}

fn parse_line(line: &[u8]) -> Result<(&str, &str), Error> {
    let line = line.strip_prefix(b"H ").ok_or(Error::InvalidSyntax)?;
    let line = str::from_utf8(line).map_err(Error::Utf8Error)?;
    line.split_once(':').ok_or(Error::InvalidSyntax)
}
