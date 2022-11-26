// Reason: everything here should be a very thin wrapper around a call to C
#![allow(unsafe_code, clippy::undocumented_unsafe_blocks)]

pub mod parser;
pub mod stream;
pub mod tools;

#[cfg(not(target_os = "linux"))]
compile_error!("blackbox-sys only supports Linux");
