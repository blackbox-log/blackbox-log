#![no_main]

use blackbox_fuzz::{encoding, fuzz_target, get_streams};

fuzz_target!(|data: &[u8]| {
    let (mut reference, mut biterator) = get_streams(data).unwrap();

    let expected = reference.read_tagged_16_v2();
    let got = encoding::read_tagged_16(LogVersion::V2, &mut biterator);

    for (expected, got) in expected.iter().zip(got.iter()) {
        assert_eq!(*expected, i64::from(*got));
    }
});
