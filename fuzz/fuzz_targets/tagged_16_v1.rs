#![no_main]

use blackbox::{encoding, LogVersion};
use blackbox_fuzz::{fuzz_target, UnalignedBytes};

fuzz_target!(|data: UnalignedBytes| {
    let (mut reference, mut biterator) = data.to_streams().unwrap();

    let expected = reference.read_tagged_16_v1();
    let got = encoding::read_tagged_16(LogVersion::V1, &mut biterator);

    if let Ok(got) = got {
        let got = got.map(Into::into);
        assert_eq!(expected, got);
    }
});
