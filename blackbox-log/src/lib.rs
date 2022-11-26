#![warn(unsafe_code, clippy::std_instead_of_alloc, clippy::std_instead_of_core)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[macro_use]
pub mod common;

pub mod betaflight;
pub mod inav;

mod file;
pub mod log;
pub mod parser;
pub mod units;

pub use self::file::File;
pub use self::log::Log;
