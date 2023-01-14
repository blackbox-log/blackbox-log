// The string data will be immediately copied out by JS, so this doesn't use
// `Borrowing`. That way it can be passed by value and doesn't need a `*_free`
// function. `wasm-bindgen` does something similar in the impl of `IntoWasmAbi`
// for `str`.
#[repr(C)]
pub struct WasmStr(usize, *const u8);

impl From<&str> for WasmStr {
    #[inline]
    fn from(s: &str) -> Self {
        Self(s.len(), s.as_ptr())
    }
}

impl From<Option<&str>> for WasmStr {
    #[inline]
    fn from(s: Option<&str>) -> Self {
        s.map_or(Self(0, std::ptr::null()), Self::from)
    }
}

// SAFETY: requires multi-value returns
unsafe impl crate::WasmSafe for WasmStr {}
