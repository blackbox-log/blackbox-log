#![allow(unsafe_code)]

mod borrowing;
mod file;
mod log;
mod str;

use std::ptr;

pub(crate) use borrowing::Borrowing;

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
pub extern "C" fn data_alloc(len: usize) -> *mut u8 {
    use std::alloc::{alloc, Layout};

    let Ok(layout) = Layout::array::<u8>(len) else { return ptr::null_mut(); };
    // SAFETY: [u8; _] is a non-zero-sized type
    unsafe { alloc(layout) }
}
