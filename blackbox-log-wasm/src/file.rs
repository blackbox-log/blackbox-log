use blackbox_log::File;

use crate::headers::WasmHeaders;
use crate::{Borrowing, OwnedSlice, WasmFfi};

pub struct WasmFile(Borrowing<File<'static>>);

impl WasmFile {
    pub(crate) fn new(data: OwnedSlice) -> Self {
        Self(Borrowing::new(data, |data| File::new(data)))
    }

    #[inline(always)]
    pub fn log_count(&self) -> usize {
        self.0.log_count()
    }

    pub fn parse_headers(&self, log: usize) -> WasmHeaders {
        let reader = self.0.new_borrow(|file| file.get_reader(log));
        WasmHeaders::new(reader)
    }
}

impl WasmFfi for WasmFile {}

#[no_mangle]
pub unsafe extern "wasm" fn file_free(ptr: *mut WasmFile) {
    let file = WasmFile::from_wasm(ptr);
    drop(file);
}

#[no_mangle]
pub unsafe extern "wasm" fn file_new(data: *mut u8, len: usize) -> *mut WasmFile {
    let data = OwnedSlice::new(data, len);
    let file = Box::new(WasmFile::new(data));
    file.into_wasm()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "wasm" fn file_logCount(ptr: *mut WasmFile) -> usize {
    let file = WasmFile::from_wasm(ptr);
    let count = file.log_count();
    file.into_wasm();
    count
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "wasm" fn file_getHeaders(ptr: *mut WasmFile, log: usize) -> *mut WasmHeaders {
    let file = WasmFile::from_wasm(ptr);
    let headers = Box::new(file.parse_headers(log));
    file.into_wasm();
    headers.into_wasm()
}
