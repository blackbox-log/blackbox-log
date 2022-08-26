#![no_main]

use blackbox_fuzz::{encoding, fuzz_target, UnalignedBytes};

fuzz_target!(|data: UnalignedBytes| {
    let (mut reference, mut biterator) = data.to_streams().unwrap();

    if let Ok(result) = encoding::read_i32_elias_delta(&mut biterator) {
        assert_eq!(reference.read_i32_elias_delta(), result);
    }
});
