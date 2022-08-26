#![no_main]

use blackbox_fuzz::{encoding, fuzz_target, get_streams};

fuzz_target!(|data: &[u8]| {
    let (mut reference, mut biterator) = get_streams(data).unwrap();

    assert_eq!(
        reference.read_ivar(),
        encoding::read_ivar(&mut biterator).unwrap_or(0)
    );
});
