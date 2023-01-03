use std::rc::Rc;

use blackbox_log::prelude::*;
use blackbox_log::Reader;

use crate::data::WasmDataParser;
use crate::str::WasmStr;
use crate::{Borrowing, WasmFfi};

pub(crate) struct WasmHeadersInner<'data> {
    headers: Headers<'data>,
    reader: Reader<'data>,
}

pub struct WasmHeaders(Rc<Borrowing<WasmHeadersInner<'static>>>);

impl WasmHeaders {
    pub(crate) fn new(reader: Borrowing<Reader<'static>>) -> Self {
        // TODO: error handling
        Self(Rc::new(reader.map(|mut reader| WasmHeadersInner {
            headers: Headers::parse(&mut reader).unwrap(),
            reader,
        })))
    }

    fn get_header<T>(&self, get: impl FnOnce(&Headers<'static>) -> T) -> Borrowing<T> {
        self.0.new_borrow(|inner| get(&inner.headers))
    }

    fn get_data_parser(&self) -> WasmDataParser {
        WasmDataParser::new(Borrowing::new_with_rc(
            Rc::clone(&self.0),
            |headers: &Borrowing<WasmHeadersInner, _>| {
                DataParser::new(headers.reader.clone(), &headers.headers)
            },
        ))
    }
}

impl WasmFfi for WasmHeaders {}

#[no_mangle]
pub unsafe extern "wasm" fn headers_free(ptr: *mut WasmHeaders) {
    let headers = WasmHeaders::from_wasm(ptr);
    drop(headers);
}

#[no_mangle]
pub unsafe extern "wasm" fn headers_getDataParser(ptr: *mut WasmHeaders) -> *mut WasmDataParser {
    let headers = WasmHeaders::from_wasm(ptr);
    let parser = headers.get_data_parser();
    headers.into_wasm();

    let parser = Box::new(parser);
    parser.into_wasm()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "wasm" fn headers_firmwareRevision(ptr: *mut WasmHeaders) -> WasmStr {
    let headers = WasmHeaders::from_wasm(ptr);
    let firmware = headers.get_header(|headers| headers.firmware_revision);
    headers.into_wasm();

    (*firmware).into()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "wasm" fn headers_boardInfo(ptr: *mut WasmHeaders) -> WasmStr {
    let headers = WasmHeaders::from_wasm(ptr);
    let info = headers.get_header(|headers| headers.board_info);
    headers.into_wasm();

    (*info).into()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "wasm" fn headers_craftName(ptr: *mut WasmHeaders) -> WasmStr {
    let headers = WasmHeaders::from_wasm(ptr);
    let name = headers.get_header(|headers| headers.craft_name);
    headers.into_wasm();

    (*name).into()
}
