#![no_main]

use blackbox_fuzz::{decoders, fuzz_target, UnalignedBytes};

fuzz_target!(|data: UnalignedBytes| {
    let (mut reference, mut bits) = data.to_streams_aligned().unwrap();

    assert_eq!(
        reference.read_uvar(),
        decoders::read_uvar(&mut bits).unwrap_or(0)
    );
});
