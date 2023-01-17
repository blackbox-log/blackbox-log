#![allow(unsafe_code)]

#[macro_use]
mod macros;

mod data;
mod ffi;
mod file;
mod headers;
mod owned_slice;
mod panic;
mod shared;
mod str;

use std::ptr;

pub(crate) use self::ffi::{FromWasmFfi, IntoWasmFfi, Structural, WasmByValue, WasmFfi};
pub(crate) use self::owned_slice::OwnedSlice;
pub(crate) use self::shared::Shared;

#[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
#[link(wasm_import_module = "main")]
extern "C" {
    fn panic(message: str::WasmStr);
    fn throw(message: str::OwnedWasmStr) -> !;
}

#[allow(clippy::undocumented_unsafe_blocks)]
#[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
fn throw_headers_parse_error(err: blackbox_log::headers::ParseError) -> ! {
    let message = err.to_string().into();
    unsafe { throw(message) };
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
fn throw_headers_parse_error(err: blackbox_log::headers::ParseError) -> ! {
    panic!("{}", err);
}

wasm_export! {
    fn data_alloc(len: owned usize) -> *mut u8 {
        OwnedSlice::<u8>::alloc(len).unwrap_or(ptr::null_mut())
    }
}
