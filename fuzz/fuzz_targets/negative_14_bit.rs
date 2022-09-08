#![no_main]

use blackbox_fuzz::{decode, fuzz_target, UnalignedBytes};

fuzz_target!(|data: UnalignedBytes| {
    let (mut reference, mut bits) = data.to_streams_aligned().unwrap();

    assert_eq!(
        reference.read_negative_14_bit(),
        decode::negative_14_bit(&mut bits).unwrap_or(0)
    );
});
