#![warn(unsafe_code, clippy::std_instead_of_alloc, clippy::std_instead_of_core)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[macro_use]
pub mod common;

pub mod betaflight;
pub mod inav;

pub mod log;
pub mod parser;
pub mod units;

use alloc::vec::Vec;

use memchr::memmem;
use tracing::instrument;

pub use self::log::Log;
use self::parser::ParseResult;

#[derive(Debug)]
pub struct File<'data> {
    offsets: Vec<usize>,
    data: &'data [u8],
}

impl<'data> File<'data> {
    pub fn new(data: &'data [u8]) -> Self {
        let offsets = memmem::find_iter(data, parser::MARKER).collect();
        Self { offsets, data }
    }

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
