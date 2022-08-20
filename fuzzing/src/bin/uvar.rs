use fuzzing::{encoding, fuzz, get_streams};

fn main() {
    loop {
        fuzz!(|bytes: &[u8]| {
            let (mut reference, mut biterator) = get_streams(bytes).unwrap();

            assert_eq!(
                reference.read_uvar(),
                encoding::read_uvar(&mut biterator).unwrap_or(0)
            );
        })
    }
}
