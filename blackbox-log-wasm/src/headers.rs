use blackbox_log::prelude::*;
use blackbox_log::Reader;

use crate::data::WasmDataParser;
use crate::str::WasmStr;
use crate::{OwnedSlice, Shared, WasmFfi};

pub struct WasmHeaders {
    headers: Shared<Headers<'static>>,
    reader: Reader<'static>,
    data: Shared<OwnedSlice>,
}

impl WasmHeaders {
    pub(crate) fn new(mut reader: Reader<'static>, data: Shared<OwnedSlice>) -> Self {
        // TODO: error handling
        let headers = Headers::parse(&mut reader).unwrap();

        Self {
            headers: Shared::new(headers),
            reader,
            data,
        }
    }

    fn get_data_parser(&self) -> WasmDataParser {
        WasmDataParser::new(
            Shared::clone(&self.headers),
            self.reader.clone(),
            Shared::clone(&self.data),
        )
    }
}

impl WasmFfi for WasmHeaders {}

#[no_mangle]
unsafe extern "C" fn headers_free(ptr: *mut WasmHeaders) {
    let headers = WasmHeaders::from_wasm(ptr);
    drop(headers);
}

#[no_mangle]
unsafe extern "C" fn headers_getDataParser(ptr: *mut WasmHeaders) -> *mut WasmDataParser {
    let headers = WasmHeaders::from_wasm(ptr);
    let parser = headers.get_data_parser();
    headers.into_wasm();

    let parser = Box::new(parser);
    parser.into_wasm()
}

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn headers_firmwareRevision(ptr: *mut WasmHeaders) -> WasmStr {
    let headers = WasmHeaders::from_wasm(ptr);
    let firmware = headers.headers.firmware_revision;
    headers.into_wasm();
    firmware.into()
}

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn headers_boardInfo(ptr: *mut WasmHeaders) -> WasmStr {
    let headers = WasmHeaders::from_wasm(ptr);
    let info = headers.headers.board_info;
    headers.into_wasm();
    info.into()
}

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn headers_craftName(ptr: *mut WasmHeaders) -> WasmStr {
    let headers = WasmHeaders::from_wasm(ptr);
    let name = headers.headers.craft_name;
    headers.into_wasm();
    name.into()
}
