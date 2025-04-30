#![expect(clippy::use_debug)]

use blackbox_log::data_v2::DataParser;
use blackbox_log::headers_v2::frame_defs::FrameDefBuilders;
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
        frame: &'a [u32],
    ) -> Self::Output {
        eprintln!("main({kind:?}): {frame:?}");
    }

    fn slow(&mut self, frame: &'a [u32]) -> Self::Output {
        eprintln!("slow: {frame:?}");
    }

    fn gps(&mut self, frame: &'a [u32]) -> Self::Output {
        eprintln!("gps: {frame:?}");
    }

    fn gps_home(&mut self, frame: &'a [u32]) -> Self::Output {
        eprintln!("gps_home: {frame:?}");
    }

    fn event(&mut self) -> Self::Output {
        eprintln!("event");
    }
}
