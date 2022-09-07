#![no_main]

use blackbox_fuzz::{decoders, fuzz_target, UnalignedBytes};

fuzz_target!(|data: UnalignedBytes| {
    let (mut reference, mut bits) = data.to_streams_aligned().unwrap();

    let expected = reference.read_tagged_32();
    let got = decoders::read_tagged_32(&mut bits);

    if let Ok(got) = got {
        let got = got.map(Into::into);
        assert_eq!(expected, got);
    }
});
