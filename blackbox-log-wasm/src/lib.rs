#![allow(unsafe_code)]

#[macro_use]
mod macros;

mod data;
mod ffi;
mod file;
mod headers;
mod owned_slice;
mod shared;
mod str;

use std::ptr;

pub(crate) use self::ffi::{FromWasmFfi, IntoWasmFfi, WasmFfi, WasmSafe};
pub(crate) use self::owned_slice::OwnedSlice;
pub(crate) use self::shared::Shared;

#[link(wasm_import_module = "main")]
extern "C" {
    fn panic(len: usize, data: *const u8);
}

wasm_export! {
    fn data_alloc(len: owned usize) -> *mut u8 {
        OwnedSlice::<u8>::alloc(len).unwrap_or(ptr::null_mut())
    }

    fn set_panic_hook() {
        std::panic::set_hook(Box::new(panic_hook));
    }
}

fn panic_hook(info: &std::panic::PanicInfo) {
    let message = info.to_string().into_boxed_str();
    let message = Box::leak(message);

    let ptr = message.as_ptr();
    let len = message.len();

    // SAFETY: the message was leaked
    unsafe { panic(len, ptr) };
}
