use super::read_uvar;
use crate::ParseResult;
use biterator::Biterator;
use std::io::Read;
use tracing::instrument;

#[instrument(level = "trace", skip(data), ret)]
pub fn read_negative_14_bit<R: Read>(data: &mut Biterator<R>) -> ParseResult<i32> {
    data.byte_align();

    // Reason: done to match reference impl
    #[allow(clippy::cast_possible_truncation)]
    let result = read_uvar(data)? as u16;
    let result = if (result & 0x2000) > 0 {
        i32::from((result | 0xC000) as i16)
    } else {
        i32::from(result)
    };

    Ok(-result)
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::case;

    #[case(0, &[0]; "zero")]
    #[case(-0x1FFF, &[0xFF, 0x3F]; "min")]
    #[case(0x2000, &[0x80, 0x40]; "max")]
    #[case(1, &[0xFF, 0x7F]; "all bits set")]
    #[case(1, &[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]; "extra bits ignored")]
    fn read_negative_14_bit(expected: i32, bytes: &[u8]) {
        let mut b = Biterator::new(bytes);
        assert_eq!(expected, super::read_negative_14_bit(&mut b).unwrap());
    }
}
