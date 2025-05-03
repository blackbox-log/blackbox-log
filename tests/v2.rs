#![expect(clippy::use_debug)]

use blackbox_log::data_v2::{DataParser, Frame};
use blackbox_log::headers_v2::frame_defs::{
    FrameDefBuilders, GpsFrameDef, GpsHomeFrameDef, MainFrameDef, SlowFrameDef,
};
use blackbox_log::headers_v2::HeadersParser;

static LOG: &[u8] = include_bytes!("./logs/error-recovery.bbl");

#[test]
fn v2() {
    let mut headers = HeadersParser::new(LOG);
    let mut frames = FrameDefBuilders::new();
    for pair in &mut headers {
        let (header, value) = pair.unwrap();
        frames.update(header, value).unwrap();

        eprintln!("{header:?}: {value:?}");
    }

    let frames = frames.build().unwrap();
    let mut data = DataParser::new(headers.data(), frames);

    while let Some(result) = data.next(Visitor) {
        result.unwrap();
    }

    todo!()
}

struct Visitor;

impl<'a> blackbox_log::data_v2::Visitor<'a> for Visitor {
    type Output = ();

    fn main(
        &mut self,
        kind: blackbox_log::data_v2::MainFrameKind,
        frame: Frame<'a, MainFrameDef>,
    ) -> Self::Output {
        eprintln!("main({kind:?}): {:?}", frame.iter().collect::<Vec<_>>());
    }

    fn slow(&mut self, frame: Frame<'a, SlowFrameDef>) -> Self::Output {
        eprintln!("slow: {:?}", frame.iter_raw().collect::<Vec<_>>());
    }

    fn gps(&mut self, frame: Frame<'a, GpsFrameDef>) -> Self::Output {
        eprintln!("gps: {:?}", frame.iter_raw().collect::<Vec<_>>());
    }

    fn gps_home(&mut self, frame: Frame<'a, GpsHomeFrameDef>) -> Self::Output {
        eprintln!("gps_home: {:?}", frame.iter_raw().collect::<Vec<_>>());
    }

    fn event(&mut self) -> Self::Output {
        eprintln!("event");
    }
}
