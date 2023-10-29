use alloc::vec::Vec;
use core::fmt;

use memchr::memmem;

use crate::headers::{Headers, ParseResult};

/// A complete blackbox log file containing zero or more logs.
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
    #[inline]
    pub fn log_count(&self) -> usize {
        self.offsets.len()
    }

    /// Returns an iterator over parsed [`Headers`] for each log.
    ///
    /// Roughly equivalent to repeatedly calling [`File::parse`], but may
    /// be able to eliminate bounds checks and skips the `Option`
    /// wrapper.
    pub fn iter(&self) -> impl Iterator<Item = ParseResult<Headers<'data>>> + '_ {
        self.offsets
            .iter()
            .map(|&offset| Headers::parse(&self.data[offset..]))
    }

    /// Attempts to parse the headers of the `index`-th log. Returns `None` if
    /// there is no log number `index`.
    pub fn parse(&self, index: usize) -> Option<ParseResult<Headers<'data>>> {
        let start = *self.offsets.get(index)?;
        let data = if let Some(&next) = self.offsets.get(index + 1) {
            &self.data[start..next]
        } else {
            &self.data[start..]
        };

        Some(Headers::parse(data))
    }
}

impl fmt::Debug for File<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File")
            .field("offsets", &self.offsets)
            .finish_non_exhaustive()
    }
}
