use alloc::vec::Vec;
use core::fmt;

use memchr::memmem;

use crate::Reader;

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

    /// Returns an iterator over [`Reader`]s for each log.
    ///
    /// Equivalent to repeatedly calling [`File::get_reader`], but may be able
    /// to eliminate bounds checks and cannot panic.
    pub fn iter(&self) -> impl Iterator<Item = Reader<'data>> + '_ {
        self.offsets
            .iter()
            .map(|&offset| Reader::new(&self.data[offset..]))
    }

    /// Returns a [`Reader`] aligned to the start of the `index`-th log.
    ///
    /// # Panics
    ///
    /// This panics if `index >= self.log_count()`.
    pub fn get_reader(&self, index: usize) -> Reader<'data> {
        let start = self.offsets[index];
        Reader::new(&self.data[start..])
    }
}

impl fmt::Debug for File<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File")
            .field("offsets", &self.offsets)
            .finish_non_exhaustive()
    }
}
