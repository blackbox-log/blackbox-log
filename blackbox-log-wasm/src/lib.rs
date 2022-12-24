#![allow(unsafe_code, missing_debug_implementations)]

use std::ptr;

use blackbox_log::{File, Headers, Log, Reader};
use buffer::Borrowing;

mod buffer {
    use std::mem;
    use std::ops::Deref;
    use std::rc::Rc;

    // SAFETY: since data is second, it will be dropped second, meaning borrower's
    // reference will not dangle
    pub(crate) struct Borrowing<T, D = Box<[u8]>> {
        borrower: T,
        #[allow(unused)]
        data: Rc<D>,
    }

    impl<T, D> Borrowing<T, D> {
        pub(crate) fn new(data: D, new: impl FnOnce(&'static D) -> T) -> Self
        where
            D: 'static,
        {
            let data = Rc::new(data);
            // SAFETY: ???
            let data_ref: &'static _ = unsafe { mem::transmute(data.deref()) };
            Self {
                borrower: new(data_ref),
                data,
            }
        }

        pub(crate) fn new_borrow<U>(&self, new: impl FnOnce(&T) -> U) -> Borrowing<U, D> {
            Borrowing {
                borrower: new(&self.borrower),
                data: self.data.clone(),
            }
        }

        pub(crate) fn map<U>(self, map: impl FnOnce(T) -> U) -> Borrowing<U, D> {
            Borrowing {
                borrower: map(self.borrower),
                data: self.data,
            }
        }
    }

    impl<T, D> Borrowing<Option<T>, D> {
        pub(crate) fn transpose(self) -> Option<Borrowing<T, D>> {
            Some(Borrowing {
                borrower: self.borrower?,
                data: self.data,
            })
        }
    }

    impl<T, D> Deref for Borrowing<T, D> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.borrower
        }
    }
}

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

#[repr(C)]
struct WasmStr {
    len: u32,
    ptr: *const u8,
}

impl WasmStr {
    unsafe fn new(s: &str) -> Self {
        let len = s.len().try_into().unwrap_or(u32::MAX);
        let ptr = s.as_ptr();
        Self { len, ptr }
    }
}

pub struct LogStr(Borrowing<WasmStr, Box<[u8]>>);

impl WasmFfi for LogStr {}

#[no_mangle]
pub unsafe extern "C" fn str_free(ptr: *mut LogStr) {
    LogStr::drop(ptr);
}

#[no_mangle]
pub extern "C" fn data_alloc(len: usize) -> *mut u8 {
    use std::alloc::{alloc, Layout};

    let Ok(layout) = Layout::array::<u8>(len) else { return ptr::null_mut(); };
    // SAFETY: [u8; _] is a non-zero-sized type
    unsafe { alloc(layout) }
}

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

pub struct WasmHeaders(Borrowing<Headers<'static>, Box<[u8]>>);

impl WasmHeaders {
    fn new(reader: Borrowing<Reader<'static>, Box<[u8]>>) -> Self {
        Self(reader.map(|mut reader| Headers::parse(&mut reader).unwrap()))
    }

    fn get_header<T>(&self, get: impl FnOnce(&Headers) -> T) -> Borrowing<T, Box<[u8]>> {
        self.0.new_borrow(get)
    }
}

impl WasmFfi for WasmHeaders {}

#[no_mangle]
pub unsafe extern "C" fn headers_free(ptr: *mut WasmHeaders) {
    let headers = WasmHeaders::from_wasm(ptr);
    drop(headers);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn headers_firmwareRevision(ptr: *mut (), is_log: bool) -> *mut LogStr {
    let get = |headers: &Headers| WasmStr::new(headers.firmware_revision);

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

    let firmware = LogStr(firmware);
    Box::new(firmware).into_wasm()
}
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn headers_boardInfo(ptr: *mut (), is_log: bool) -> *mut LogStr {
    let get = |headers: &Headers| headers.board_info.map(|s| WasmStr::new(s));

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

    info.transpose()
        .map_or(ptr::null_mut(), |name| Box::new(LogStr(name)).into_wasm())
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn headers_craftName(ptr: *mut (), is_log: bool) -> *mut LogStr {
    let get = |headers: &Headers| headers.craft_name.map(|s| WasmStr::new(s));

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

    name.transpose()
        .map_or(ptr::null_mut(), |name| Box::new(LogStr(name)).into_wasm())
}

pub struct WasmLog(Borrowing<Log<'static>, Box<[u8]>>);

impl WasmLog {
    fn new(reader: Borrowing<Reader<'static>, Box<[u8]>>) -> Self {
        Self(reader.map(|mut reader| Log::parse(&mut reader).unwrap()))
    }

    fn get_header<T>(&self, get: impl FnOnce(&Headers) -> T) -> Borrowing<T, Box<[u8]>> {
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
