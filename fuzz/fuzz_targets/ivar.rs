#![no_main]

use blackbox_fuzz::{encoding, fuzz_target, UnalignedBytes};

fuzz_target!(|data: UnalignedBytes| {
    let (mut reference, mut bits) = data.to_streams_aligned().unwrap();

    assert_eq!(
        reference.read_ivar(),
        encoding::read_ivar(&mut bits).unwrap_or(0)
    );
});
