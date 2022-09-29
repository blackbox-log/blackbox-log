#![warn(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[macro_use]
pub mod common;

pub mod betaflight;
pub mod inav;

pub mod parser;

pub use common::{DisarmReason, DisarmReasonError};

use alloc::vec::Vec;
use core::str::FromStr;
use core::{slice, str};
use memchr::memmem;
use parser::{Config, Data, Event, Headers, MainFrame, ParseResult, Reader, SlowFrame};
use tracing::instrument;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogVersion {
    V1,
    V2,
}

impl FromStr for LogVersion {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "1" | "v1" => Ok(Self::V1),
            "2" | "v2" => Ok(Self::V2),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Log<'data> {
    headers: Headers<'data>,
    data: Data,
}

impl<'data> Log<'data> {
    pub fn parse<'a>(config: &'a Config, data: &'data [u8]) -> ParseResult<Self> {
        let mut data = Reader::new(data);
        let headers = Headers::parse(&mut data)?;
        let data = Data::parse(&mut data, config, &headers)?;

        Ok(Self { headers, data })
    }

    pub fn headers(&self) -> &Headers<'data> {
        &self.headers
    }

    pub fn events(&self) -> &[Event] {
        &self.data.events
    }

    pub fn main_frames(&self) -> &[MainFrame] {
        &self.data.main_frames
    }

    // pub fn gps_frames(&self) -> &[Frame] {
    //     &self.data.gps_frames
    // }

    // pub fn gps_home_frames(&self) -> &[Frame] {
    //     &self.data.gps_home_frames
    // }

    pub fn slow_frames(&self) -> &[SlowFrame] {
        &self.data.slow_frames
    }
}

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

    pub fn parse_iter<'config, 'file>(
        &'file self,
        config: &'config Config,
    ) -> FileIter<'config, 'file, 'data> {
        FileIter {
            config,
            offsets: self.offsets.iter(),
            file: self,
        }
    }

    /// # Panics
    ///
    /// This panics if given an `index` greater than or equal to the number of logs in the file.
    pub fn parse_index(&self, config: &Config, index: usize) -> ParseResult<Log<'data>> {
        let start = self.offsets[index];
        self.parse_offset(config, start)
    }

    #[instrument(level = "trace", skip(self, config))]
    fn parse_offset(&self, config: &Config, start: usize) -> ParseResult<Log<'data>> {
        Log::parse(config, &self.data[start..])
    }
}

pub struct FileIter<'config, 'file, 'data> {
    config: &'config Config,
    offsets: slice::Iter<'file, usize>,
    file: &'file File<'data>,
}

impl<'config, 'file, 'data> Iterator for FileIter<'config, 'file, 'data> {
    type Item = ParseResult<Log<'data>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.offsets
            .next()
            .map(|&start| self.file.parse_offset(self.config, start))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.offsets.size_hint()
    }
}
