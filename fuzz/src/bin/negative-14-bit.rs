#![no_main]

use blackbox_fuzz::{decode, fuzz_target, AlignedBytes};

fuzz_target!(|data: AlignedBytes| {
    let (mut reference, mut bits) = data.to_streams().unwrap();

    assert_eq!(
        reference.read_negative_14_bit(),
        decode::negative_14_bit(&mut bits).unwrap_or(0)
    );
});
