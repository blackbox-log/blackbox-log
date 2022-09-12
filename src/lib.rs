#![warn(unsafe_code)]

pub mod betaflight;
pub mod parser;

use parser::{Data, Event, Frame, Headers};
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
pub struct Log {
    headers: Headers,
    data: Data,
}

impl Log {
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn events(&self) -> &[Event] {
        &self.data.events
    }

    pub fn main_frames(&self) -> &[Frame] {
        &self.data.main_frames
    }

    pub fn gps_frames(&self) -> &[Frame] {
        &self.data.gps_frames
    }

    pub fn gps_home_frames(&self) -> &[Frame] {
        &self.data.gps_home_frames
    }

    pub fn slow_frames(&self) -> &[Frame] {
        &self.data.slow_frames
    }
}
