use biterator::Biterator;
use std::io::Read;
use tracing::instrument;

#[instrument(level = "trace", skip(data), ret)]
pub fn read_negative_14_bit<R: Read>(data: &mut Biterator<R>) -> i32 {
    data.byte_align();

    unimplemented!();
}
