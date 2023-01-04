use blackbox_log::prelude::*;
use blackbox_log::Reader;

use crate::{OwnedSlice, Shared, WasmFfi};

// TODO: use Headers as borrowee
pub struct WasmDataParser {
    parser: DataParser<'static, 'static>,
    _headers: Shared<Headers<'static>>,
    _data: Shared<OwnedSlice>,
}

impl WasmDataParser {
    pub(crate) fn new(
        headers: Shared<Headers<'static>>,
        reader: Reader<'static>,
        data: Shared<OwnedSlice>,
    ) -> Self {
        let headers_ref = unsafe { headers.deref_static() };

        Self {
            parser: DataParser::new(reader, headers_ref),
            _headers: headers,
            _data: data,
        }
    }

    fn main_frame_count(&self) -> usize {
        self.parser.stats().counts.main
    }

    fn gps_frame_count(&self) -> usize {
        self.parser.stats().counts.gps
    }
}

impl WasmFfi for WasmDataParser {}

#[no_mangle]
pub unsafe extern "wasm" fn data_free(ptr: *mut WasmDataParser) {
    let parser = WasmDataParser::from_wasm(ptr);
    drop(parser);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "wasm" fn data_mainFrameCount(ptr: *mut WasmDataParser) -> usize {
    let parser = WasmDataParser::from_wasm(ptr);
    let count = parser.main_frame_count();
    parser.into_wasm();
    count
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "wasm" fn data_gpsFrameCount(ptr: *mut WasmDataParser) -> usize {
    let parser = WasmDataParser::from_wasm(ptr);
    let count = parser.gps_frame_count();
    parser.into_wasm();
    count
}
