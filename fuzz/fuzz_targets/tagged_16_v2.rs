#![no_main]

use blackbox::{encoding, LogVersion};
use blackbox_fuzz::{fuzz_target, get_streams};

fuzz_target!(|data: &[u8]| {
    let (mut reference, mut biterator) = get_streams(data).unwrap();

    let expected = reference.read_tagged_16_v2();
    let got = encoding::read_tagged_16(LogVersion::V2, &mut biterator);

    if let Ok(got) = got {
        let got = got.map(Into::into);
        assert_eq!(expected, got);
    }
});
