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
        let _ = self.data.read_u8();

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

#[cfg(test)]
mod tests {
    use super::*;

    fn header<'n, 'v>(
        name: &'n [u8],
        value: &'v [u8],
    ) -> Result<(&'n [u8], &'v [u8]), MissingColonError> {
        Ok((name, value))
    }

    #[test]
    fn valid() {
        let mut headers = HeadersParser::new(b"H Test:true\nH :empty header\nH Empty value:\nbody");
        assert_eq!(header(b"Test", b"true"), headers.next().unwrap());
        assert_eq!(header(b"", b"empty header"), headers.next().unwrap());
        assert_eq!(header(b"Empty value", b""), headers.next().unwrap());
        assert!(headers.next().is_none());
        assert_eq!(Some(b"body".as_slice()), headers.data.read_line());
    }

    #[test]
    fn corrupted() {
        let mut headers = HeadersParser::new(&[
            72, 32, 72, 97, 115, 58, 110, 117, 0, 108, 108, 10, 72, 32, 78, 111, 110, 45, 117, 116,
            102, 56, 58, 0, 159, 146, 150, 10, 98, 111, 100, 121,
        ]);
        assert_eq!(header(b"Has", b"nu\0ll"), headers.next().unwrap());
        assert_eq!(
            header(b"Non-utf8", &[0, 159, 146, 150]),
            headers.next().unwrap()
        );
        assert!(headers.next().is_none());
    }
}
