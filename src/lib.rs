#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::std_instead_of_alloc, clippy::std_instead_of_core)]
#![warn(unreachable_pub, clippy::missing_panics_doc)]

//! Ergonomic parser for Betaflight blackbox logs.
//!
//! For details about the format of blackbox logs, see the *Blackbox Internals*
//! development documentation from [INAV][inav-doc], [Betaflight][bf-doc]
//!
//! # Examples
//!
//! The simplest way to extract a few fields of interest:
//!
//! ```
//! use blackbox_log::frame::FieldDef;
//! use blackbox_log::prelude::*;
//!
//! let filters = blackbox_log::FieldFilterSet {
//!     // This restricts the included fields to `loopIteration`, `time` and
//!     // `rcCommand[0]` through `rcCommand[3]` for main frames
//!     main: Some(["rcCommand"].into()),
//!     // ... and only `flightModeFlags` for slow frames
//!     slow: Some(["flightModeFlags"].into()),
//!     gps: None,
//! };
//!
//! let file = b"...";
//! for mut reader in blackbox_log::File::new(file).iter() {
//!     let headers = Headers::parse(&mut reader).unwrap();
//!
//!     let mut parser = DataParser::with_filters(reader, &headers, &filters);
//!     while let Some(event) = parser.next() {
//!         match event {
//!             ParserEvent::Main(main) => {
//!                 for (value, FieldDef { name, .. }) in
//!                     main.iter().zip(headers.main_frame_def().iter())
//!                 {
//!                     println!("{name}: {value:?}");
//!                 }
//!             }
//!             ParserEvent::Slow(slow) => {
//!                 for (value, FieldDef { name, .. }) in
//!                     slow.iter().zip(headers.slow_frame_def().iter())
//!                 {
//!                     println!("{name}: {value:?}");
//!                 }
//!             }
//!             ParserEvent::Event(_) | ParserEvent::Gps(_) => {}
//!         }
//!     }
//! }
//! ```
//!
//! Get only the GPS data without parsing logs that cannot contain GPS frames:
//!
//! ```
//! use blackbox_log::frame::FieldDef;
//! use blackbox_log::prelude::*;
//!
//! let file = b"...";
//! let file = blackbox_log::File::new(file);
//!
//! for mut reader in file.iter() {
//!     let headers = Headers::parse(&mut reader).unwrap();
//!
//!     if let Some(gps_def) = &headers.gps_frame_def() {
//!         let mut parser = DataParser::new(reader, &headers);
//!
//!         while let Some(event) = parser.next() {
//!             if let ParserEvent::Gps(gps) = event {
//!                 for (value, FieldDef { name, .. }) in gps.iter().zip(gps_def.iter()) {
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
//!   [`std::error::Error`] for [`headers::ParseError`].
//!
//! [bf-doc]: https://betaflight.com/docs/development/Blackbox%20Internals
//! [inav-doc]: https://github.com/iNavFlight/inav/blob/master/docs/development/Blackbox%20Internals.md

extern crate alloc;

#[macro_use]
mod utils;

pub mod data;
pub mod event;
mod file;
mod filter;
pub mod frame;
pub mod headers;
mod parser;
mod predictor;
pub mod prelude;
mod reader;
pub mod units;

use core::ops::Range;

pub use self::data::{DataParser, ParserEvent};
pub use self::event::Event;
pub use self::file::File;
pub use self::filter::{FieldFilter, FieldFilterSet};
pub use self::frame::{Unit, Value};
use self::headers::FirmwareVersion;
pub use self::headers::Headers;
pub use self::reader::Reader;

/// The first line of any blackbox log.
const MARKER: &[u8] = b"H Product:Blackbox flight data recorder by Nicholas Sherlock\n";

const BETAFLIGHT_SUPPORT: Range<FirmwareVersion> =
    FirmwareVersion::new(4, 2, 0)..FirmwareVersion::new(4, 5, 0);
const INAV_SUPPORT: Range<FirmwareVersion> =
    FirmwareVersion::new(5, 0, 0)..FirmwareVersion::new(5, 2, 0);
