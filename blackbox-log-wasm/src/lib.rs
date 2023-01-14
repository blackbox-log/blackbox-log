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

pub(crate) use self::ffi::{FromWasmFfi, IntoWasmFfi, WasmFfi, WasmSafe};
pub(crate) use self::owned_slice::OwnedSlice;
pub(crate) use self::shared::Shared;

wasm_export! {
    fn data_alloc(len: owned usize) -> *mut u8 {
        OwnedSlice::<u8>::alloc(len).unwrap_or(ptr::null_mut())
    }
}
