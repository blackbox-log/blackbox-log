#![allow(unsafe_code)]

#[macro_use]
mod macros;

mod data;
mod file;
mod headers;
mod owned_slice;
mod shared;
mod str;

use std::ptr;

pub(crate) use self::owned_slice::OwnedSlice;
pub(crate) use self::shared::Shared;

unsafe trait WasmSafe {}

unsafe impl WasmSafe for () {}
unsafe impl WasmSafe for u32 {}
unsafe impl WasmSafe for i32 {}
unsafe impl WasmSafe for u64 {}
unsafe impl WasmSafe for i64 {}
unsafe impl WasmSafe for f32 {}
unsafe impl WasmSafe for f64 {}
unsafe impl<T> WasmSafe for *const T {}
unsafe impl<T> WasmSafe for *mut T {}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
unsafe impl WasmSafe for usize {}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
unsafe impl<const N: usize, T: WasmSafe> WasmSafe for [T; N] {}

trait WasmFfi {
    type Ffi: WasmSafe;
}

impl<T: WasmSafe> WasmFfi for T {
    type Ffi = T;
}

trait IntoWasmFfi: WasmFfi {
    fn into_ffi(self) -> Self::Ffi;
}

impl<T: WasmSafe> IntoWasmFfi for T {
    #[inline(always)]
    fn into_ffi(self) -> Self {
        self
    }
}

trait FromWasmFfi: WasmFfi {
    unsafe fn from_ffi(ffi: Self::Ffi) -> Self;
}

impl<T: WasmSafe> FromWasmFfi for T {
    #[inline(always)]
    unsafe fn from_ffi(ffi: Self::Ffi) -> Self {
        ffi
    }
}

wasm_export! {
    fn data_alloc(len: owned usize) -> *mut u8 {
        OwnedSlice::<u8>::alloc(len).unwrap_or(ptr::null_mut())
    }
}

#[link(wasm_import_module = "main")]
extern "C" {
    fn panic(len: usize, data: *const u8);
}

wasm_export! {
    fn set_panic_hook() {
        std::panic::set_hook(Box::new(panic_hook));
    }
}

fn panic_hook(info: &std::panic::PanicInfo) {
    let message = info.to_string();

    let ptr = message.as_ptr();
    let len = message.len();

    unsafe { panic(len, ptr) };
}
