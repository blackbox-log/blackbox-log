use fuzzing::{encoding, fuzz, get_streams};

fn main() {
    loop {
        fuzz!(|bytes: &[u8]| {
            let (mut reference, mut biterator) = get_streams(bytes).unwrap();

            assert_eq!(
                reference.read_u32_elias_delta(),
                encoding::read_u32_elias_delta(&mut biterator)
            );
        })
    }
}
