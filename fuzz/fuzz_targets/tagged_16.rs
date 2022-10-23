#![no_main]

use blackbox_fuzz::{decode, fuzz_target, AlignedBytes};

fuzz_target!(|data: AlignedBytes| {
    let (mut reference, mut bits) = data.to_streams().unwrap();

    let expected = reference.read_tagged_16_v2();
    let got = decode::tagged_16(&mut bits);

    if let Ok(got) = got {
        let got = got.map(Into::into);
        assert_eq!(expected, got);
    }
});
