use core::iter::FusedIterator;

use crate::Reader;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MissingColonError;

impl fmt::Display for MissingColonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("missing colon between header name and value")
    }
}

impl core::error::Error for MissingColonError {}

#[derive(Debug, Clone)]
pub struct HeadersParser<'data> {
    data: Reader<'data>,
}

impl<'data> HeadersParser<'data> {
    pub fn new(data: &'data [u8]) -> Self {
        Self {
            data: Reader::new(data),
        }
    }
}

impl<'data> Iterator for HeadersParser<'data> {
    type Item = Result<(&'data [u8], &'data [u8]), MissingColonError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.peek() != Some(b'H') {
            return None;
        }

        // cannot be empty after peek() is Some
        let line = self.data.read_line().unwrap();
        let line = line.strip_prefix(b" ").unwrap_or(line);

        let Some(colon) = line.iter().position(|&x| x == b':') else {
            return Some(Err(MissingColonError));
        };
        Some(Ok((&line[0..colon], &line[(colon + 1)..])))
    }
}

impl FusedIterator for HeadersParser<'_> {}
