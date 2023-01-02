#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::std_instead_of_alloc, clippy::std_instead_of_core)]
#![warn(unreachable_pub, clippy::missing_panics_doc)]

//! Ergonomic parser for Betaflight blackbox logs.
//!
//! For details about the format of blackbox logs, see the *Blackbox Internals*
//! development documentation from [INAV][inav-doc], [Betaflight][bf-doc], or
//! [EmuFlight][emu-doc].
//!
//! # Examples
//!
//! The simplest way to extract a few fields of interest:
//!
//! ```
//! use blackbox_log::prelude::*;
//!
//! let file = b"...";
//! for mut reader in blackbox_log::File::new(file).iter() {
//!     let headers = Headers::parse(&mut reader).unwrap();
//!     let mut parser = DataParser::new(&mut reader, &headers);
//!     // let mut view = log.data();
//!
//!     // This restricts the included fields to `time` and `rcCommand[0]` through `rcCommand[3]`
//!     // TODO: view.update_filter(&["time", "rcCommand"]);
//!
//!     while let Some(event) = parser.next() {
//!         match event {
//!             ParseEvent::Main(main) => {
//!                 for (value, (name, _)) in main.iter().zip(headers.main_def().iter()) {
//!                     println!("{name}: {value:?}");
//!                 }
//!             }
//!             ParseEvent::Slow(slow) => {
//!                 for (value, (name, _)) in slow.iter().zip(headers.slow_def().iter()) {
//!                     println!("{name}: {value:?}");
//!                 }
//!             }
//!             ParseEvent::Event(_) | ParseEvent::Gps(_) => {}
//!         }
//!     }
//! }
//! ```
//!
//! Get only the GPS data without parsing logs that cannot contain GPS frames:
//!
//! ```
//! use blackbox_log::prelude::*;
//!
//! let file = b"...";
//! let file = blackbox_log::File::new(file);
//!
//! for mut reader in file.iter() {
//!     let headers = Headers::parse(&mut reader).unwrap();
//!
//!     if let Some(gps_def) = headers.gps_def() {
//!         let mut parser = DataParser::new(&mut reader, &headers);
//!
//!         while let Some(event) = parser.next() {
//!             if let ParseEvent::Gps(gps) = event {
//!                 for (value, name) in gps.iter().zip(gps_def.iter_names()) {
//!                     println!("{name}: {value:?}");
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # Features
//!
//! - `std`: **Enabled** by default. Currently, this only implements
//!   [`std::error::Error`] for [`HeadersParseError`].
//! - `serde`: **Disabled** by default. This allows serializing parsed logs
//!   using `serde`. **Note:** This is only used for snapshot testing and is not
//!   yet intended for public use.
//!
//! [bf-doc]: https://github.com/betaflight/betaflight/blob/master/docs/development/Blackbox%20Internals.md
//! [inav-doc]: https://github.com/iNavFlight/inav/blob/master/docs/development/Blackbox%20Internals.md
//! [emu-doc]: https://github.com/EmuFlight/emuflight/blob/master/docs/development/Blackbox%20Internals.md

extern crate alloc;

#[macro_use]
mod utils;

pub mod data;
pub mod event;
mod file;
pub mod frame;
pub mod headers;
mod predictor;
pub mod prelude;
mod reader;
pub mod units;

#[cfg(any(bench, fuzzing))]
pub mod parser;
#[cfg(not(any(bench, fuzzing)))]
mod parser;

pub use self::data::{DataParser, ParseEvent};
pub use self::file::File;
pub use self::frame::{Unit, Value};
pub use self::headers::{
    Headers, ParseError as HeadersParseError, ParseResult as HeadersParseResult,
};
pub use self::reader::Reader;

/// The first line of any blackbox log.
const MARKER: &[u8] = b"H Product:Blackbox flight data recorder by Nicholas Sherlock\n";
