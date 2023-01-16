use std::mem::ManuallyDrop;
use std::pin::Pin;

use blackbox_log::data::FrameCounts;
use blackbox_log::frame::{Frame, GpsFrame, MainFrame, SlowFrame};
use blackbox_log::prelude::*;
use blackbox_log::Reader;

use crate::{OwnedSlice, Shared};

// SAFETY: field order *must* be `parser` first, then `headers`, then `data` to
// ensure correct drop order
pub struct WasmDataParser {
    parsed: Pin<Box<WasmParseEvent>>,
    parser: DataParser<'static, 'static>,
    _headers: Shared<Headers<'static>>,
    _data: Shared<OwnedSlice<u8>>,
}

impl_boxed_wasm_ffi!(WasmDataParser);

impl WasmDataParser {
    pub(crate) fn new(
        headers: Shared<Headers<'static>>,
        reader: Reader<'static>,
        data: Shared<OwnedSlice<u8>>,
    ) -> Self {
        // SAFETY: this is only used to create the `DataParser`, which is guaranteed to
        // be dropped before `headers` by the declaration order in the struct
        let headers_ref = unsafe { headers.deref_static() };

        Self {
            parsed: Box::pin(WasmParseEvent::default()),
            parser: DataParser::new(reader, headers_ref),
            _headers: headers,
            _data: data,
        }
    }

    fn result_ptr(&self) -> *const WasmParseEvent {
        let parsed: &WasmParseEvent = &self.parsed;
        parsed
    }

    fn frame_counts(&self) -> FrameCounts {
        self.parser.stats().counts
    }

    fn next(&mut self) {
        let parsed = self.parser.next();
        *self.parsed = parsed.into();
    }
}

#[repr(C)]
pub struct WasmParseEvent {
    kind: WasmParseEventKind,
    data: WasmParseEventData,
}

#[repr(u8)]
enum WasmParseEventKind {
    None = 0,
    Event,
    Main,
    Slow,
    Gps,
}

#[repr(C)]
union WasmParseEventData {
    none: (),
    event: (),
    main: ManuallyDrop<DataMain>,
    slow: ManuallyDrop<DataSlow>,
    gps: ManuallyDrop<DataGps>,
}

#[repr(transparent)]
struct Fields(OwnedSlice<u32>);

#[repr(C)]
struct DataMain {
    fields: Fields,
    iteration: u32,
    time: WasmDuration,
}

#[repr(C)]
struct DataSlow {
    fields: Fields,
}

#[repr(C)]
struct DataGps {
    fields: Fields,
    time: WasmDuration,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
struct WasmDuration {
    microseconds: u16,
    milliseconds: u16,
    seconds: u8,
    minutes: u8,
    hours: u8,
}

impl Drop for WasmParseEvent {
    fn drop(&mut self) {
        use {WasmParseEventData as Data, WasmParseEventKind as Kind};

        // SAFETY: data & kind get set together and will always match
        unsafe {
            #[allow(clippy::unneeded_field_pattern)]
            #[allow(clippy::match_same_arms)]
            match (&self.kind, &mut self.data) {
                (Kind::None, Data { none: _ }) => {}
                (Kind::Event, Data { event: _ }) => {}
                (Kind::Main, Data { main }) => ManuallyDrop::drop(main),
                (Kind::Slow, Data { slow }) => ManuallyDrop::drop(slow),
                (Kind::Gps, Data { gps }) => ManuallyDrop::drop(gps),
            }
        }
    }
}

impl Default for WasmParseEvent {
    fn default() -> Self {
        Self {
            kind: WasmParseEventKind::None,
            data: WasmParseEventData { none: () },
        }
    }
}

impl From<Option<ParserEvent<'_, '_, '_>>> for WasmParseEvent {
    fn from(event: Option<ParserEvent>) -> Self {
        let Some(event) = event else {
            return Self::default();
        };

        match event {
            ParserEvent::Event(_) => Self {
                kind: WasmParseEventKind::Event,
                data: WasmParseEventData { event: () },
            },
            ParserEvent::Main(main) => Self {
                kind: WasmParseEventKind::Main,
                data: WasmParseEventData {
                    main: ManuallyDrop::new(main.into()),
                },
            },
            ParserEvent::Slow(slow) => Self {
                kind: WasmParseEventKind::Slow,
                data: WasmParseEventData {
                    slow: ManuallyDrop::new(slow.into()),
                },
            },
            ParserEvent::Gps(gps) => Self {
                kind: WasmParseEventKind::Gps,
                data: WasmParseEventData {
                    gps: ManuallyDrop::new(gps.into()),
                },
            },
        }
    }
}

impl From<MainFrame<'_, '_, '_>> for DataMain {
    fn from(frame: MainFrame) -> Self {
        let iteration = frame.iteration();
        let time = WasmDuration::from_microseconds(frame.time_raw());

        Self {
            fields: Fields::from(frame),
            iteration,
            time,
        }
    }
}

impl From<SlowFrame<'_, '_>> for DataSlow {
    fn from(frame: SlowFrame) -> Self {
        Self {
            fields: Fields::from(frame),
        }
    }
}

impl From<GpsFrame<'_, '_>> for DataGps {
    fn from(frame: GpsFrame) -> Self {
        let time = WasmDuration::from_microseconds(frame.time_raw());

        Self {
            fields: Fields::from(frame),
            time,
        }
    }
}

impl<F: Frame> From<F> for Fields {
    fn from(frame: F) -> Self {
        let mut slice = OwnedSlice::new_zeroed(frame.len());

        for (i, out) in slice.iter_mut().enumerate() {
            *out = frame.get_raw(i).unwrap();
        }

        Self(slice)
    }
}

impl WasmDuration {
    fn from_microseconds(us: u64) -> Self {
        const US_PER_MS: u64 = 1000;
        const MS_PER_SEC: u64 = 1000;
        const SEC_PER_MIN: u64 = 60;
        const MIN_PER_HOUR: u64 = 60;

        #[allow(clippy::cast_possible_truncation)]
        let new = |hours, min, sec, ms, us| Self {
            microseconds: us as u16,
            milliseconds: ms as u16,
            seconds: sec as u8,
            minutes: min as u8,
            hours: hours as u8,
        };

        let ms = us / US_PER_MS;
        let sec = ms / MS_PER_SEC;
        let min = sec / SEC_PER_MIN;
        let hours = min / MIN_PER_HOUR;

        // Saturate at the full 10hr 16min max...
        if hours > u8::MAX.into() {
            return new(
                u8::MAX.into(),
                MIN_PER_HOUR - 1,
                SEC_PER_MIN - 1,
                MS_PER_SEC - 1,
                US_PER_MS - 1,
            );
        }

        new(
            hours,
            min % MIN_PER_HOUR,
            sec % SEC_PER_MIN,
            ms % MS_PER_SEC,
            us % US_PER_MS,
        )
    }
}

wasm_export!(free data_free: Box<WasmDataParser>);
wasm_export! {
    fn data_resultPtr(parser: ref Box<WasmDataParser>) -> *const WasmParseEvent {
        parser.result_ptr()
    }

    fn data_counts(parser: ref Box<WasmDataParser>) -> [usize; 5] {
        let counts = parser.frame_counts();

        [
            counts.event,
            counts.main,
            counts.slow,
            counts.gps,
            counts.gps_home,
        ]
    }

    fn data_next(parser: ref_mut Box<WasmDataParser>) {
        parser.next();
    }
}
