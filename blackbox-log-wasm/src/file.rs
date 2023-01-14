use blackbox_log::File;

use crate::headers::WasmHeaders;
use crate::{OwnedSlice, Shared};

// SAFETY: `file` *must* stay before `data` to ensure correct drop order
pub struct WasmFile {
    file: File<'static>,
    data: Shared<OwnedSlice>,
}

impl_boxed_wasm_ffi!(WasmFile);

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

wasm_export!(free file_free: Box<WasmFile>);
wasm_export! {
    fn file_new(data: owned *mut u8, len: owned usize) -> Box<WasmFile> {
        let data = OwnedSlice::new(data, len);
        Box::new(WasmFile::new(data))
    }

    fn file_logCount(file: ref Box<WasmFile>) -> usize {
        file.log_count()
    }

    fn file_getHeaders(file: ref Box<WasmFile>, log: owned usize) -> Box<WasmHeaders> {
        Box::new(file.parse_headers(log))
    }
}
