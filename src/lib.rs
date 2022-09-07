#![warn(unsafe_code)]

pub mod betaflight;
pub mod parser;

use bitter::BigEndianReader;
use bitter::BitReader;
use parser::{Data, Headers};
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

pub(crate) type Reader<'a> = BigEndianReader<'a>;

// Reason: unfinished
#[allow(dead_code)]
#[derive(Debug)]
pub struct Log {
    headers: Headers,
    data: Data,
}

pub(crate) fn byte_align(data: &mut Reader) {
    while !data.byte_aligned() {
        data.consume(1);
    }
}
