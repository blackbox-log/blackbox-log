use alloc::vec::Vec;

use memchr::memmem;
use tracing::instrument;

use crate::parser::ParseResult;
use crate::Log;

/// Represents a complete blackbox log file containing zero or more logs.
#[derive(Debug)]
pub struct File<'data> {
    offsets: Vec<usize>,
    data: &'data [u8],
}

impl<'data> File<'data> {
    /// Creates a new `File` from a byte slice of its contents.
    ///
    /// This is relatively cheap, since it only scans for log start markers
    /// without parsing any data.
    pub fn new(data: &'data [u8]) -> Self {
        let offsets = memmem::find_iter(data, crate::MARKER).collect();
        Self { offsets, data }
    }

    /// Returns the number of log start markers in the file.
    pub fn log_count(&self) -> usize {
        self.offsets.len()
    }

    /// Returns an iterator over all parsed [`Log`]s in the file.
    ///
    /// The logs are parsed lazily --- no work will be done until
    /// `Iterator::next` is called.
    pub fn parse_iter<'a>(&'a self) -> impl Iterator<Item = ParseResult<Log<'data>>> + 'a {
        (0..self.log_count()).map(|i| self.parse_by_index(i))
    }

    /// Parses a [`Log`] by index.
    ///
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
