#![no_main]

use blackbox_fuzz::{encoding, fuzz_target, UnalignedBytes};

fuzz_target!(|data: UnalignedBytes| {
    let (mut reference, mut biterator) = data.to_streams().unwrap();

    let expected = reference.read_tagged_32();
    let got = encoding::read_tagged_32(&mut biterator);

    if let Ok(got) = got {
        let got = got.map(Into::into);
        assert_eq!(expected, got);
    }
});
