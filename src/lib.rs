#![no_std]
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
//! use blackbox_log::Filter;
//!
//! let filters = blackbox_log::FilterSet {
//!     // This restricts the included fields to `rcCommand[0]` through `rcCommand[3]`
//!     main: Filter::OnlyFields(["rcCommand"].into()),
//!     // ... only `flightModeFlags` for slow frames
//!     slow: Filter::OnlyFields(["flightModeFlags"].into()),
//!     // ... and no filter for gps frames -- include all fields
//!     gps: Filter::Unfiltered,
//! };
//!
//! let file = b"...";
//! for headers in blackbox_log::File::new(file).iter() {
//!     let headers = headers.expect("valid log headers");
//!
//!     let mut parser = headers.data_parser_with_filters(&filters);
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
//! for headers in blackbox_log::File::new(file).iter() {
//!     let headers = headers.expect("valid log headers");
//!
//!     if let Some(gps_def) = &headers.gps_frame_def() {
//!         let mut parser = headers.data_parser();
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
//! [bf-doc]: https://betaflight.com/docs/development/Blackbox-Internals
//! [inav-doc]: https://github.com/iNavFlight/inav/blob/master/docs/development/Blackbox%20Internals.md

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

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
pub use self::filter::{FieldFilter, Filter, FilterSet};
pub use self::frame::{Unit, Value};
use self::headers::FirmwareVersion;
pub use self::headers::Headers;
use self::reader::Reader;

/// The first line of any blackbox log.
const MARKER: &[u8] = b"H Product:Blackbox flight data recorder by Nicholas Sherlock\n";

const BETAFLIGHT_SUPPORT: Range<FirmwareVersion> =
    FirmwareVersion::new(4, 2, 0)..FirmwareVersion::new(4, 5, 0);
const INAV_SUPPORT: &[Range<FirmwareVersion>] = &[
    FirmwareVersion::new(5, 0, 0)..FirmwareVersion::new(5, 2, 0),
    FirmwareVersion::new(6, 0, 0)..FirmwareVersion::new(6, 2, 0),
    FirmwareVersion::new(7, 0, 0)..FirmwareVersion::new(7, 1, 0),
];
