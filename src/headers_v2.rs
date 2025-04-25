use core::iter::FusedIterator;
use core::str;

use crate::Reader;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Utf8Error(#[from] str::Utf8Error),
    #[error("missing colon between header name and value")]
    MissingColon,
}

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
    type Item = Result<(&'data str, &'data str), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.peek() != Some(b'H') {
            return None;
        }
        let _ = self.data.read_u8();

        // cannot be empty after peek() is Some
        let line = self.data.read_line().unwrap();
        let line = line.strip_prefix(b" ").unwrap_or(line);

        Some(match str::from_utf8(line) {
            Ok(line) => line.split_once(':').ok_or(Error::MissingColon),
            Err(err) => Err(Error::Utf8Error(err)),
        })
    }
}

impl FusedIterator for HeadersParser<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    fn header<'n, 'v>(name: &'n str, value: &'v str) -> Result<(&'n str, &'v str), Error> {
        Ok((name, value))
    }

    #[test]
    fn valid() {
        let mut headers = HeadersParser::new(b"H Test:true\nH :empty header\nH Empty value:\nbody");
        assert_eq!(header("Test", "true"), headers.next().unwrap());
        assert_eq!(header("", "empty header"), headers.next().unwrap());
        assert_eq!(header("Empty value", ""), headers.next().unwrap());
        assert!(headers.next().is_none());
        assert_eq!(Some(b"body".as_slice()), headers.data.read_line());
    }

    #[test]
    fn corrupted() {
        let mut headers = HeadersParser::new(&[
            72, 32, 72, 97, 115, 58, 110, 117, 0, 108, 108, 10, 72, 32, 78, 111, 110, 45, 117, 116,
            102, 56, 58, 0, 159, 146, 150, 10, 98, 111, 100, 121,
        ]);
        assert_eq!(header("Has", "nu\0ll"), headers.next().unwrap());
        assert!(matches!(
            headers.next().unwrap().unwrap_err(),
            Error::Utf8Error(_)
        ));
        assert!(headers.next().is_none());
    }
}
