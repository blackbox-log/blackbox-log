#![no_main]

use blackbox_fuzz::{fuzz_target, get_streams};
use blackbox::{encoding, LogVersion};

fuzz_target!(|data: &[u8]| {
    let (mut reference, mut biterator) = get_streams(data).unwrap();

    let expected = reference.read_tagged_16_v1();
    let got = encoding::read_tagged_16(LogVersion::V1, &mut biterator);

    for (expected, got) in expected.iter().zip(got.iter()) {
        assert_eq!(*expected, i64::from(*got));
    }
});
