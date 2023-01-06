use blackbox_log::File;

use crate::headers::WasmHeaders;
use crate::{OwnedSlice, Shared, WasmFfi};

// SAFETY: `file` *must* stay before `data` to ensure correct drop order
pub struct WasmFile {
    file: File<'static>,
    data: Shared<OwnedSlice>,
}

impl WasmFile {
    pub(crate) fn new(data: OwnedSlice) -> Self {
        let data = Shared::new(data);

        // SAFETY: this is only used to create the `File`, which is guaranteed to be
        // dropped before `data` by the declaration order in the struct
        let data_ref = unsafe { data.deref_static() };

        Self {
            file: File::new(data_ref),
            data,
        }
    }

    #[inline(always)]
    pub fn log_count(&self) -> usize {
        self.file.log_count()
    }

    pub fn parse_headers(&self, log: usize) -> WasmHeaders {
        let reader = self.file.get_reader(log);
        WasmHeaders::new(reader, Shared::clone(&self.data))
    }
}

impl WasmFfi for WasmFile {}

#[no_mangle]
unsafe extern "wasm" fn file_free(ptr: *mut WasmFile) {
    let file = WasmFile::from_wasm(ptr);
    drop(file);
}

#[no_mangle]
unsafe extern "wasm" fn file_new(data: *mut u8, len: usize) -> *mut WasmFile {
    let data = OwnedSlice::new(data, len);
    let file = Box::new(WasmFile::new(data));
    file.into_wasm()
}

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "wasm" fn file_logCount(ptr: *mut WasmFile) -> usize {
    let file = WasmFile::from_wasm(ptr);
    let count = file.log_count();
    file.into_wasm();
    count
}

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "wasm" fn file_getHeaders(ptr: *mut WasmFile, log: usize) -> *mut WasmHeaders {
    let file = WasmFile::from_wasm(ptr);
    let headers = Box::new(file.parse_headers(log));
    file.into_wasm();
    headers.into_wasm()
}
