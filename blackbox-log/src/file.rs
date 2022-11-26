use alloc::vec::Vec;

use memchr::memmem;
use tracing::instrument;

use crate::parser::ParseResult;
use crate::Log;

#[derive(Debug)]
pub struct File<'data> {
    offsets: Vec<usize>,
    data: &'data [u8],
}

impl<'data> File<'data> {
    pub fn new(data: &'data [u8]) -> Self {
        let offsets = memmem::find_iter(data, crate::MARKER).collect();
        Self { offsets, data }
    }

    /// Returns the number of log start markers found
    pub fn log_count(&self) -> usize {
        self.offsets.len()
    }

    /// Returns an iterator over all the (lazily) parsed logs in the file
    pub fn parse_iter<'a>(&'a self) -> impl Iterator<Item = ParseResult<Log<'data>>> + 'a {
        (0..self.log_count()).map(|i| self.parse_by_index(i))
    }

    /// # Panics
    ///
    /// This panics if given an `index` greater than or equal to the number of
    /// logs in the file.
    #[instrument(level = "trace", skip(self), fields(offset))]
    pub fn parse_by_index(&self, index: usize) -> ParseResult<Log<'data>> {
        let start = self.offsets[index];
        tracing::Span::current().record("offset", start);

        Log::parse(&self.data[start..])
    }
}