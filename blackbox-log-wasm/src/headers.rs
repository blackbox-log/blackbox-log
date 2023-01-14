use core::slice;
use std::alloc::{self, Layout};
use std::ptr;

use blackbox_log::frame::{FieldDef, FrameDef};
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

    fn main_def(&self) -> WasmFrameDef {
        WasmFrameDef::from(&self.headers.main_frame_def)
    }

    fn slow_def(&self) -> WasmFrameDef {
        WasmFrameDef::from(&self.headers.slow_frame_def)
    }

    fn gps_def(&self) -> WasmFrameDef {
        self.headers
            .gps_frame_def
            .as_ref()
            .map_or_else(WasmFrameDef::default, WasmFrameDef::from)
    }
}

impl WasmFfi for WasmHeaders {}

#[repr(C)]
struct WasmFrameDef {
    // TODO: generic OwnedSlice
    len: usize,
    data: *mut WasmFieldDef,
}

impl WasmFfi for WasmFrameDef {}

#[repr(C)]
struct WasmFieldDef {
    name_len: usize,
    name_data: *const u8,
    signed: bool,
}

impl WasmFrameDef {
    #[inline]
    fn layout(len: usize) -> Option<Layout> {
        if len == 0 {
            return None;
        }

        Some(Layout::array::<WasmFieldDef>(len).unwrap())
    }
}

impl Default for WasmFrameDef {
    fn default() -> Self {
        Self {
            len: 0,
            data: ptr::null_mut(),
        }
    }
}

impl Drop for WasmFrameDef {
    fn drop(&mut self) {
        if self.data.is_null() {
            return;
        }

        if let Some(layout) = Self::layout(self.len) {
            unsafe { alloc::dealloc(self.data as *mut u8, layout) }
        }

        self.data = ptr::null_mut();
    }
}

impl<'data, F: FrameDef<'data>> From<&F> for WasmFrameDef {
    fn from(frame: &F) -> Self {
        let len = frame.len();
        let layout = Self::layout(len).unwrap();

        let data = unsafe { alloc::alloc_zeroed(layout) as *mut WasmFieldDef };

        let slice: &mut [WasmFieldDef] = unsafe { slice::from_raw_parts_mut(data, len) };

        for (i, out) in slice.iter_mut().enumerate() {
            let FieldDef { name, signed, .. } = frame.get(i).unwrap();

            *out = WasmFieldDef {
                name_len: name.len(),
                name_data: name.as_ptr(),
                signed,
            };
        }

        Self { len, data }
    }
}

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
unsafe extern "C" fn frameDef_free(ptr: *mut WasmFrameDef) {
    let def = WasmFrameDef::from_wasm(ptr);
    drop(def);
}

#[no_mangle]
unsafe extern "C" fn headers_mainDef(ptr: *mut WasmHeaders) -> *mut WasmFrameDef {
    let headers = WasmHeaders::from_wasm(ptr);
    let def = headers.main_def();
    headers.into_wasm();

    Box::new(def).into_wasm()
}

#[no_mangle]
unsafe extern "C" fn headers_slowDef(ptr: *mut WasmHeaders) -> *mut WasmFrameDef {
    let headers = WasmHeaders::from_wasm(ptr);
    let def = headers.slow_def();
    headers.into_wasm();

    Box::new(def).into_wasm()
}

#[no_mangle]
unsafe extern "C" fn headers_gpsDef(ptr: *mut WasmHeaders) -> *mut WasmFrameDef {
    let headers = WasmHeaders::from_wasm(ptr);
    let def = headers.gps_def();
    headers.into_wasm();

    Box::new(def).into_wasm()
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
