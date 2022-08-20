use blackbox::LogVersion;
use fuzzing::{encoding, fuzz, get_streams};

fn main() {
    loop {
        fuzz!(|bytes: &[u8]| {
            let (mut reference, mut biterator) = get_streams(bytes).unwrap();

            let expected = reference.read_tagged_16_v2();
            let got = encoding::read_tagged_16(LogVersion::V2, &mut biterator);

            for (expected, got) in expected.iter().zip(got.iter()) {
                assert_eq!(*expected, i64::from(*got));
            }
        })
    }
}
