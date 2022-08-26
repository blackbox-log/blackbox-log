#![no_main]

use blackbox_fuzz::{encoding, fuzz_target, get_streams};

fuzz_target!(|data: &[u8]| {
    let (mut reference, mut biterator) = get_streams(data).unwrap();

    if let Ok(result) = encoding::read_u32_elias_delta(&mut biterator) {
        assert_eq!(reference.read_u32_elias_delta(), result);
    }
});
