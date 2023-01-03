use blackbox_log::prelude::*;

use crate::headers::WasmHeadersInner;
use crate::{Borrowing, WasmFfi};

// TODO: use Headers as borrowee
pub struct WasmDataParser(
    Borrowing<DataParser<'static, 'static>, Borrowing<WasmHeadersInner<'static>>>,
);

impl WasmDataParser {
    pub(crate) fn new(
        inner: Borrowing<DataParser<'static, 'static>, Borrowing<WasmHeadersInner<'static>>>,
    ) -> Self {
        Self(inner)
    }

    fn main_frame_count(&self) -> usize {
        self.0.stats().counts.main
    }

    fn gps_frame_count(&self) -> usize {
        self.0.stats().counts.gps
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
