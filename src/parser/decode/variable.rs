use super::zig_zag_decode;
use crate::parser::{ParseError, ParseResult};
use crate::Reader;
use bitter::BitReader;

#[allow(clippy::assertions_on_constants)]
pub fn variable(data: &mut Reader) -> ParseResult<u32> {
    // 32 bits at 7 bits / byte = 5 bytes
    const _: () = assert!((5 * 8) <= bitter::MAX_READ_BITS, "bit buffer is too small");

    debug_assert!(data.byte_aligned());

    data.refill_lookahead();

    let mut uvar: u32 = 0;
    let mut offset: u32 = 0;
    loop {
        if !data.has_bits_remaining(8) {
            return Err(ParseError::UnexpectedEof);
        }

        let is_last_byte = data.peek(1) == 0;
        data.consume(1);

        uvar |= (data.peek(7) << offset) as u32;
        data.consume(7);

        offset += 7;

        if !is_last_byte && offset >= 32 {
            return Err(ParseError::Corrupted);
        }

        if is_last_byte {
            break;
        }
    }

    Ok(uvar)
}

pub fn variable_signed(data: &mut Reader) -> ParseResult<i32> {
    variable(data).map(zig_zag_decode)
}

#[cfg(test)]
mod test {
    use super::*;

    fn read_ok(bytes: &[u8]) -> u32 {
        variable(&mut Reader::new(bytes)).unwrap()
    }

    #[test]
    fn zero() {
        assert_eq!(0, read_ok(&[0x00]));
        assert_eq!(0, read_ok(&[0x80, 0x00]));
    }

    #[test]
    fn one() {
        assert_eq!(1, read_ok(&[1]));
    }

    #[test]
    fn full_byte_output() {
        assert_eq!(0xFF, read_ok(&[0xFF, 0x01]));
    }

    #[test]
    fn max_two_byte_input() {
        assert_eq!(0x3FFF, read_ok(&[0xFF, 0x7F]));
    }

    #[test]
    fn max() {
        assert_eq!(0xFFFF_FFFF, read_ok(&[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]));
    }

    #[test]
    #[should_panic(expected = "Corrupted")]
    fn too_many_bytes() {
        assert_eq!(0xFFFF_FFFF, read_ok(&[0x80, 0x80, 0x80, 0x80, 0x80]));
    }
}
