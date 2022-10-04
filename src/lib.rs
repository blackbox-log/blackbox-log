#![warn(unsafe_code, clippy::std_instead_of_alloc, clippy::std_instead_of_core)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[macro_use]
pub mod common;

pub mod betaflight;
pub mod inav;

pub mod parser;

use alloc::vec::Vec;
use core::iter;

use memchr::memmem;
use tracing::instrument;

pub use self::common::{DisarmReason, DisarmReasonError, LogVersion};
use self::parser::{Config, Data, Event, Headers, MainFrame, ParseResult, Reader, SlowFrame};
use crate::parser::Frame;

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

    /// # Panics
    ///
    /// This panics if given an `index` greater than or equal to the number of
    /// logs in the file.
    #[instrument(level = "trace", skip(self, config), fields(offset))]
    pub fn parse_by_index<'config>(
        &self,
        config: &'config Config,
        index: usize,
    ) -> ParseResult<Log<'data>> {
        tracing::trace!(?config);

        let start = self.offsets[index];
        tracing::Span::current().record("offset", start);

        Log::parse(config, &self.data[start..])
    }
}

#[derive(Debug)]
pub struct Log<'data> {
    headers: Headers<'data>,
    data: Data,
}

impl<'data> Log<'data> {
    pub fn parse<'config>(config: &'config Config, data: &'data [u8]) -> ParseResult<Self> {
        let mut data = Reader::new(data);
        let headers = Headers::parse(&mut data)?;

        let data = Data::parse(data, config, &headers)?;

        Ok(Self { headers, data })
    }

    pub fn headers(&self) -> &Headers<'data> {
        &self.headers
    }

    pub fn events(&self) -> &[Event] {
        &self.data.events
    }

    pub fn iter_frames(&self) -> FrameIter {
        FrameIter {
            log: self,
            index: 0,
        }
    }

    pub fn field_names(&self) -> impl Iterator<Item = &str> {
        self.headers.main_fields().chain(self.headers.slow_fields())
    }
}

#[derive(Debug)]
pub struct FrameIter<'a, 'data> {
    log: &'a Log<'data>,
    index: usize,
}

impl<'log, 'data> Iterator for FrameIter<'log, 'data> {
    type Item = FieldIter<'log>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.log.data.main_frames.len() {
            return None;
        }

        let (main, slow) = &self.log.data.main_frames[self.index];
        let slow = &self.log.data.slow_frames[*slow];
        self.index += 1;

        Some(FieldIter {
            main,
            slow,
            field: 0,
        })
    }
}

#[derive(Debug)]
pub struct FieldIter<'a> {
    main: &'a MainFrame,
    slow: &'a SlowFrame,
    field: usize,
}

impl<'a> Iterator for FieldIter<'a> {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        let main_len = self.main.len();
        let slow_len = self.slow.len();

        let next = match self.field {
            0 => self.main.iteration().into(),
            1 => self.main.time(),
            i if i < main_len => self.main.values()[i - 2],
            i if i < (main_len + slow_len) => self.slow.values()[i - main_len],
            _ => {
                return None;
            }
        };

        self.field += 1;
        Some(next)
    }
}

impl<'a> iter::FusedIterator for FieldIter<'a> {}
