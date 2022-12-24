use crate::borrowing::Borrowing;
use crate::WasmFfi;

#[repr(C)]
pub(crate) struct WasmStr {
    len: u32,
    ptr: *const u8,
}

pub struct LogStr(Borrowing<WasmStr, Box<[u8]>>);

impl LogStr {
    pub(crate) fn new(s: Borrowing<&str, Box<[u8]>>) -> Self {
        Self(s.map(|s| {
            let len = s.len().try_into().unwrap_or(u32::MAX);
            let ptr = s.as_ptr();
            WasmStr { len, ptr }
        }))
    }
}

impl WasmFfi for LogStr {}

#[no_mangle]
pub unsafe extern "C" fn str_free(ptr: *mut LogStr) {
    LogStr::drop(ptr);
}
