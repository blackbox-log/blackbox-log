#![no_main]

use blackbox::LogVersion;
use blackbox_fuzz::{decode, fuzz_target, UnalignedBytes};

fuzz_target!(|data: UnalignedBytes| {
    let (mut reference, mut bits) = data.to_streams_aligned().unwrap();

    let expected = reference.read_tagged_16_v1();
    let got = decode::tagged_16(LogVersion::V1, &mut bits);

    if let Ok(got) = got {
        let got = got.map(Into::into);
        assert_eq!(expected, got);
    }
});
