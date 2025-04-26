use blackbox_log::headers_v2::frame_defs::FrameDefBuilders;
use blackbox_log::headers_v2::HeadersParser;

static LOG: &[u8] = include_bytes!("./logs/error-recovery.bbl");

#[test]
fn v2() {
    let mut frames = FrameDefBuilders::new();
    for pair in HeadersParser::new(LOG) {
        let (header, value) = pair.unwrap();
        frames.update(header, value).unwrap();

        eprintln!("{header:?}: {value:?}");
    }

    dbg!(frames.main.build());
    dbg!(frames.slow.build());
    dbg!(frames.gps.build());
    dbg!(frames.gps_home.build());
    todo!()
}
