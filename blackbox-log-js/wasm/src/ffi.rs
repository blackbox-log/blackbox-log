/// Types that can be read as bytes from memory.
///
/// # Safety
///
/// This should only be implemented for types with a known field layout.
pub(crate) unsafe trait Structural {}

/// Types that can be passed into/returned from an exported WebAssembly by
/// value.
///
/// # Safety
///
/// This should only be implemented for types that can be represented as one or
/// more native WebAssembly values.
pub(crate) unsafe trait WasmByValue {}

// SAFETY: Anything that can be passed by value using native types can be read
// from memory
unsafe impl<T: WasmByValue> Structural for T {}

#[allow(clippy::undocumented_unsafe_blocks)]
mod wasm_safe_impl {
    use super::*;

    // Raw bytes that aren't native WebAssembly types:
    unsafe impl Structural for u8 {}
    unsafe impl Structural for i8 {}
    unsafe impl Structural for u16 {}
    unsafe impl Structural for i16 {}
    unsafe impl Structural for u128 {}
    unsafe impl Structural for i128 {}

    // Native WebAssembly scalar types:
    unsafe impl WasmByValue for () {}
    unsafe impl WasmByValue for u32 {}
    unsafe impl WasmByValue for i32 {}
    unsafe impl WasmByValue for u64 {}
    unsafe impl WasmByValue for i64 {}
    unsafe impl WasmByValue for f32 {}
    unsafe impl WasmByValue for f64 {}

    // Raw pointers can be passed as u32/u64, but the value they point to might not
    // be readable from outside wasm
    unsafe impl<T> WasmByValue for *const T {}
    unsafe impl<T> WasmByValue for *mut T {}

    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
    unsafe impl WasmByValue for usize {}

    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
    unsafe impl<const N: usize, T: WasmByValue> WasmByValue for [T; N] {}
}

pub(crate) trait WasmFfi {
    type Ffi: WasmByValue;
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

impl<T: WasmByValue> WasmFfi for T {
    type Ffi = T;
}

impl<T: WasmByValue> IntoWasmFfi for T {
    /// Passes through a value that has the same Rust & FFI representations
    #[inline(always)]
    fn into_ffi(self) -> Self {
        self
    }
}

impl<T: WasmByValue> FromWasmFfi for T {
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
