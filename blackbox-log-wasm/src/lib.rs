#![feature(wasm_abi)]
#![allow(unsafe_code)]

mod data;
mod file;
mod headers;
mod owned_slice;
mod shared;
mod str;

use std::ptr;

pub(crate) use self::owned_slice::OwnedSlice;
pub(crate) use self::shared::Shared;

trait WasmFfi {
    unsafe fn from_wasm(ptr: *mut Self) -> Box<Self> {
        Box::from_raw(ptr)
    }

    fn into_wasm(self: Box<Self>) -> *mut Self {
        Box::into_raw(self)
    }

    unsafe fn drop(ptr: *mut Self) {
        drop(Box::from_raw(ptr));
    }
}

#[no_mangle]
pub unsafe extern "wasm" fn data_alloc(len: usize) -> *mut u8 {
    OwnedSlice::alloc(len).unwrap_or(ptr::null_mut())
}
