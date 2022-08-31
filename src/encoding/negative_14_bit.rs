use super::{read_uvar, sign_extend};
use crate::{ParseError, ParseResult};
use biterator::{Biterator, CustomInt};
use std::io::Read;
use tracing::instrument;

#[instrument(level = "trace", skip(data), ret)]
pub fn read_negative_14_bit<R: Read>(data: &mut Biterator<R>) -> ParseResult<i32> {
    data.byte_align();

    let result = read_uvar(data)? as u16;
    let result = if (result & 0x2000) > 0 {
        i32::from((result | 0xC000) as i16)
    } else {
        result as i32
    };

    Ok(-result)
}
