#![feature(wasm_abi)]
#![allow(unsafe_code)]

mod borrowing;
mod data;
mod file;
mod headers;
mod owned_slice;
mod str;

use std::ptr;

pub(crate) use borrowing::Borrowing;
pub(crate) use owned_slice::OwnedSlice;

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
