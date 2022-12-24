use std::ptr;

use blackbox_log::File;

use crate::log::{WasmHeaders, WasmLog};
use crate::{Borrowing, WasmFfi};

pub struct WasmFile(Borrowing<File<'static>, Box<[u8]>>);

impl WasmFile {
    pub fn new(data: Box<[u8]>) -> Self {
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

    pub fn parse_log(&self, log: usize) -> WasmLog {
        let reader = self.0.new_borrow(|file| file.get_reader(log));
        WasmLog::new(reader)
    }
}

impl WasmFfi for WasmFile {}

#[no_mangle]
pub unsafe extern "C" fn file_free(ptr: *mut WasmFile) {
    let file = WasmFile::from_wasm(ptr);
    drop(file);
}

#[no_mangle]
pub unsafe extern "C" fn file_new(data: *mut u8, len: usize) -> *mut WasmFile {
    let data = ptr::slice_from_raw_parts_mut(data, len);
    let data = Box::from_raw(data);
    let file = Box::new(WasmFile::new(data));
    file.into_wasm()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn file_logCount(ptr: *mut WasmFile) -> usize {
    let file = WasmFile::from_wasm(ptr);
    let count = file.log_count();
    file.into_wasm();
    count
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn file_getHeaders(ptr: *mut WasmFile, log: usize) -> *mut WasmHeaders {
    let file = WasmFile::from_wasm(ptr);
    let headers = Box::new(file.parse_headers(log));
    file.into_wasm();
    headers.into_wasm()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn file_getLog(ptr: *mut WasmFile, log: usize) -> *mut WasmLog {
    let file = WasmFile::from_wasm(ptr);
    let log = Box::new(file.parse_log(log));
    file.into_wasm();
    log.into_wasm()
}
