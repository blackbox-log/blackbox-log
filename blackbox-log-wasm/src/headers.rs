use blackbox_log::frame::{FieldDef, FrameDef};
use blackbox_log::prelude::*;
use blackbox_log::Reader;

use crate::data::WasmDataParser;
use crate::str::WasmStr;
use crate::{OwnedSlice, Shared};

pub struct WasmHeaders {
    headers: Shared<Headers<'static>>,
    reader: Reader<'static>,
    data: Shared<OwnedSlice<u8>>,
}

impl_boxed_wasm_ffi!(WasmHeaders);

impl WasmHeaders {
    pub(crate) fn new(mut reader: Reader<'static>, data: Shared<OwnedSlice<u8>>) -> Self {
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

#[derive(Default)]
#[repr(transparent)]
struct WasmFrameDef(OwnedSlice<WasmFieldDef>);

impl_boxed_wasm_ffi!(WasmFrameDef);

#[repr(C)]
struct WasmFieldDef {
    name_len: usize,
    name_data: *const u8,
    signed: bool,
}

impl<'data, F: FrameDef<'data>> From<&F> for WasmFrameDef {
    fn from(frame: &F) -> Self {
        let mut slice = OwnedSlice::new_zeroed(frame.len());

        for (i, out) in slice.iter_mut().enumerate() {
            let FieldDef { name, signed, .. } = frame.get(i).unwrap();

            *out = WasmFieldDef {
                name_len: name.len(),
                name_data: name.as_ptr(),
                signed,
            };
        }

        Self(slice)
    }
}

wasm_export!(free headers_free: Box<WasmHeaders>);
wasm_export!(free frameDef_free: Box<WasmFrameDef>);
wasm_export! {
    fn headers_getDataParser(headers: ref Box<WasmHeaders>) -> Box<WasmDataParser> {
        Box::new(headers.get_data_parser())
    }

    fn headers_mainDef(headers: ref Box<WasmHeaders>) -> Box<WasmFrameDef> {
        Box::new(headers.main_def())
    }

    fn headers_slowDef(headers: ref Box<WasmHeaders>) -> Box<WasmFrameDef> {
        Box::new(headers.slow_def())
    }

    fn headers_gpsDef(headers: ref Box<WasmHeaders>) -> Box<WasmFrameDef> {
        Box::new(headers.gps_def())
    }

    fn headers_firmwareRevision(headers: ref Box<WasmHeaders>) -> WasmStr {
        headers.headers.firmware_revision.into()
    }

    fn headers_boardInfo(headers: ref Box<WasmHeaders>) -> WasmStr {
        headers.headers.board_info.into()
    }

    fn headers_craftName(headers: ref Box<WasmHeaders>) -> WasmStr {
        headers.headers.craft_name.into()
    }
}
