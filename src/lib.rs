#![warn(unsafe_code)]

pub mod betaflight;
pub mod parser;

use memchr::memmem;
use parser::{Config, Data, Event, Headers, MainFrame, ParseResult, Reader, SlowFrame};
use std::str;
use std::str::FromStr;

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

pub fn parse_file<'a, 'data: 'a>(
    config: &'a Config,
    data: &'data [u8],
) -> impl Iterator<Item = ParseResult<Log<'data>>> + 'a {
    memmem::find_iter(data, parser::MARKER).map(|start| Log::parse(config, &data[start..]))
}
