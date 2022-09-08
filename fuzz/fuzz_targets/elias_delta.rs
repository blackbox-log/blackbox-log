#![no_main]

use blackbox_fuzz::{decode, fuzz_target, UnalignedBytes};

fuzz_target!(|data: UnalignedBytes| {
    let (mut reference, mut bits) = data.to_streams_unaligned().unwrap();

    if let Ok(result) = decode::elias_delta(&mut bits) {
        assert_eq!(reference.read_u32_elias_delta(), result);
    }
});
