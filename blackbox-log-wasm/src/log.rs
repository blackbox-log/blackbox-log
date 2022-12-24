use std::ptr;

use blackbox_log::{Headers, Log, Reader};

use crate::str::LogStr;
use crate::{Borrowing, WasmFfi};

pub struct WasmHeaders(Borrowing<Headers<'static>, Box<[u8]>>);

impl WasmHeaders {
    pub(crate) fn new(reader: Borrowing<Reader<'static>, Box<[u8]>>) -> Self {
        Self(reader.map(|mut reader| Headers::parse(&mut reader).unwrap()))
    }

    fn get_header<T>(&self, get: impl FnOnce(&Headers<'static>) -> T) -> Borrowing<T, Box<[u8]>> {
        self.0.new_borrow(get)
    }
}

impl WasmFfi for WasmHeaders {}

#[no_mangle]
pub unsafe extern "C" fn headers_free(ptr: *mut WasmHeaders) {
    let headers = WasmHeaders::from_wasm(ptr);
    drop(headers);
}

pub struct WasmLog(Borrowing<Log<'static>, Box<[u8]>>);

impl WasmLog {
    pub(crate) fn new(reader: Borrowing<Reader<'static>, Box<[u8]>>) -> Self {
        Self(reader.map(|mut reader| Log::parse(&mut reader).unwrap()))
    }

    fn get_header<T>(&self, get: impl FnOnce(&Headers<'static>) -> T) -> Borrowing<T, Box<[u8]>> {
        self.0.new_borrow(|log| get(log.headers()))
    }

    fn main_frame_count(&self) -> usize {
        self.0.stats().counts.main
    }

    fn gps_frame_count(&self) -> usize {
        self.0.stats().counts.gps
    }
}

impl WasmFfi for WasmLog {}

#[no_mangle]
pub unsafe extern "C" fn log_free(ptr: *mut WasmLog) {
    let log = WasmLog::from_wasm(ptr);
    drop(log);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn headers_firmwareRevision(ptr: *mut (), is_log: bool) -> *mut LogStr {
    let get = |headers: &Headers<'static>| headers.firmware_revision;

    let firmware = if is_log {
        let log = WasmLog::from_wasm(ptr as *mut _);
        let firmware = log.get_header(get);
        log.into_wasm();
        firmware
    } else {
        let headers = WasmHeaders::from_wasm(ptr as *mut _);
        let firmware = headers.get_header(get);
        headers.into_wasm();
        firmware
    };

    let firmware = LogStr::new(firmware);
    Box::new(firmware).into_wasm()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn headers_boardInfo(ptr: *mut (), is_log: bool) -> *mut LogStr {
    let get = |headers: &Headers<'static>| headers.board_info;

    let info = if is_log {
        let log = WasmLog::from_wasm(ptr as *mut _);
        let info = log.get_header(get);
        log.into_wasm();
        info
    } else {
        let headers = WasmHeaders::from_wasm(ptr as *mut _);
        let info = headers.get_header(get);
        headers.into_wasm();
        info
    };

    info.transpose().map_or(ptr::null_mut(), |name| {
        Box::new(LogStr::new(name)).into_wasm()
    })
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn headers_craftName(ptr: *mut (), is_log: bool) -> *mut LogStr {
    let get = |headers: &Headers<'static>| headers.craft_name;

    let name = if is_log {
        let log = WasmLog::from_wasm(ptr as *mut _);
        let name = log.get_header(get);
        log.into_wasm();
        name
    } else {
        let headers = WasmHeaders::from_wasm(ptr as *mut _);
        let name = headers.get_header(get);
        headers.into_wasm();
        name
    };

    name.transpose().map_or(ptr::null_mut(), |name| {
        Box::new(LogStr::new(name)).into_wasm()
    })
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn log_mainFrameCount(ptr: *mut WasmLog) -> usize {
    let log = WasmLog::from_wasm(ptr);
    let count = log.main_frame_count();
    log.into_wasm();
    count
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn log_gpsFrameCount(ptr: *mut WasmLog) -> usize {
    let log = WasmLog::from_wasm(ptr);
    let count = log.gps_frame_count();
    log.into_wasm();
    count
}
