#![no_main]

use blackbox_fuzz::{decode, fuzz_target, AlignedBytes};

fuzz_target!(|data: AlignedBytes| {
    let (mut reference, mut bits) = data.to_streams().unwrap();

    let expected = reference.read_tagged_32();
    let got = decode::tagged_32(&mut bits);

    if let Some(got) = got {
        let got = got.map(Into::into);
        assert_eq!(expected, got);
    }
});
