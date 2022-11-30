#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::std_instead_of_alloc, clippy::std_instead_of_core)]
#![warn(unreachable_pub, clippy::missing_panics_doc)]

extern crate alloc;

#[macro_use]
mod utils;

pub mod data;
pub mod event;
mod file;
pub mod frame;
pub mod headers;
pub mod log;
pub mod parser;
mod predictor;
mod reader;
pub mod units;

pub use self::file::File;
pub use self::frame::{GpsUnit, GpsValue, Unit, Value};
pub use self::headers::{
    Headers, ParseError as HeadersParseError, ParseResult as HeadersParseResult,
};
pub use self::log::Log;
pub use self::reader::Reader;

const MARKER: &[u8] = b"H Product:Blackbox flight data recorder by Nicholas Sherlock\n";
