#[allow(clippy::missing_safety_doc)]
/// An unsafe marker trait for types that are safe to pass by value into or out
/// of WebAssembly.
pub(crate) unsafe trait WasmSafe {}

#[allow(clippy::undocumented_unsafe_blocks)]
mod wasm_safe_impl {
    use super::*;

    // Native WebAssembly scalar types:
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
}

pub(crate) trait WasmFfi {
    type Ffi: WasmSafe;
}

pub(crate) trait IntoWasmFfi: WasmFfi {
    /// Converts a value into its WebAssembly FFI-safe representation.
    fn into_ffi(self) -> Self::Ffi;
}

pub(crate) trait FromWasmFfi: WasmFfi {
    /// Creates a native Rust type from its WebAssembly FFI-safe representation.
    ///
    /// # Safety
    ///
    /// See concrete implementation.
    unsafe fn from_ffi(ffi: Self::Ffi) -> Self;
}

impl<T: WasmSafe> WasmFfi for T {
    type Ffi = T;
}

impl<T: WasmSafe> IntoWasmFfi for T {
    /// Passes through a value that has the same Rust & FFI representations
    #[inline(always)]
    fn into_ffi(self) -> Self {
        self
    }
}

impl<T: WasmSafe> FromWasmFfi for T {
    /// Passes through a value that has the same Rust & FFI representations
    ///
    /// # Safety
    ///
    /// This is always safe.
    #[inline(always)]
    unsafe fn from_ffi(ffi: Self::Ffi) -> Self {
        ffi
    }
}
