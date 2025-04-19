use core::iter::FusedIterator;
use core::str;

use crate::Reader;

#[derive(Debug, thiserror::Error)]
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
