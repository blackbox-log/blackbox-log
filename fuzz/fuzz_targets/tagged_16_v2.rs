#![no_main]

use blackbox::{encoding, LogVersion};
use blackbox_fuzz::{fuzz_target, UnalignedBytes};

fuzz_target!(|data: UnalignedBytes| {
    let (mut reference, mut bits) = data.to_streams_aligned().unwrap();

    let expected = reference.read_tagged_16_v2();
    let got = encoding::read_tagged_16(LogVersion::V2, &mut bits);

    if let Ok(got) = got {
        let got = got.map(Into::into);
        assert_eq!(expected, got);
    }
});
