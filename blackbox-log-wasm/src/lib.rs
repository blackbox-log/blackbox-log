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
unsafe extern "C" fn data_alloc(len: usize) -> *mut u8 {
    OwnedSlice::alloc(len).unwrap_or(ptr::null_mut())
}

#[link(wasm_import_module = "main")]
extern "C" {
    fn panic(len: usize, data: *const u8);
}

#[no_mangle]
extern "C" fn set_panic_hook() {
    std::panic::set_hook(Box::new(panic_hook));
}

fn panic_hook(info: &std::panic::PanicInfo) {
    let message = info.to_string();

    let ptr = message.as_ptr();
    let len = message.len();

    unsafe { panic(len, ptr) };
}
