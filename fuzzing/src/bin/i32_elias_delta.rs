use fuzzing::{encoding, fuzz, get_streams};

fn main() {
    loop {
        fuzz!(|bytes: &[u8]| {
            let (mut reference, mut biterator) = get_streams(bytes).unwrap();

            assert_eq!(
                reference.read_i32_elias_delta(),
                encoding::read_i32_elias_delta(&mut biterator)
            );
        })
    }
}
