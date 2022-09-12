#![no_main]

use blackbox_fuzz::{decode, fuzz_target, AlignedBytes};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;

#[derive(Debug)]
struct Input {
    count: u8,
    data: AlignedBytes,
}

impl<'a> Arbitrary<'a> for Input {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let count = u.int_in_range(1..=8)?;
        let data = u.arbitrary()?;

        Ok(Self { count, data })
    }
}

fuzz_target!(|input: Input| {
    let Input { count, data } = input;

    let (mut reference, mut bits) = data.to_streams().unwrap();

    let expected = reference.read_tagged_variable(count.into());
    let got = decode::tagged_variable(&mut bits, (count - 1).into());

    if let Ok(got) = got {
        let got = got.map(Into::into);
        assert_eq!(expected, got);
    }
});
