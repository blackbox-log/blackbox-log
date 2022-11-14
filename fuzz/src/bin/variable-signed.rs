#![no_main]

use blackbox_fuzz::{decode, fuzz_target, AlignedBytes};

fuzz_target!(|data: AlignedBytes| {
    let (mut reference, mut bits) = data.to_streams().unwrap();

    assert_eq!(
        reference.read_variable_signed(),
        decode::variable_signed(&mut bits).unwrap_or(0)
    );
});
