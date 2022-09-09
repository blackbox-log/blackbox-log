use super::sign_extend;
use crate::parser::{ParseError, ParseResult};
use crate::Reader;
use bitter::BitReader;

const COUNT: usize = 3;

#[allow(clippy::assertions_on_constants)]
pub fn tagged_32(data: &mut Reader) -> ParseResult<[i32; COUNT]> {
    // Allows up to the 6 bit case in one refill
    const _: () = assert!(24 <= bitter::MAX_READ_BITS, "bit buffer is too small");

    debug_assert!(data.byte_aligned());

    let mut result = [0; COUNT];

    if data.refill_lookahead() < 8 {
        return Err(ParseError::UnexpectedEof);
    }

    let tag = data.peek(2);
    data.consume(2);

    match tag {
        0 => {
            for x in &mut result {
                *x = next_as_i32(data, 2);
            }
        }

        1 => {
            // Re-align to nibbles
            data.consume(2);

            if !data.has_bits_remaining(12) {
                return Err(ParseError::UnexpectedEof);
            }

            for x in &mut result {
                *x = next_as_i32(data, 4);
            }
        }

        2 => {
            result[0] = next_as_i32(data, 6);

            if !data.has_bits_remaining(16) {
                return Err(ParseError::UnexpectedEof);
            }

            // Skip upper 2 bits
            data.consume(2);
            result[1] = next_as_i32(data, 6);

            data.consume(2);
            result[2] = next_as_i32(data, 6);
        }

        3.. => {
            let mut tags = data.read_bits(6).unwrap();
            for x in &mut result {
                let tag = tags & 3;
                tags >>= 2;

                *x = match tag {
                    0 => data
                        .read_bits(8)
                        .map(|x| sign_extend(x, 8) as i32)
                        .ok_or(ParseError::UnexpectedEof)?,
                    1 => {
                        let value = data.read_i16().ok_or(ParseError::UnexpectedEof)?;
                        i16::from_be(value).into()
                    }

                    2 => {
                        let value: u64 = data.read_bits(24).ok_or(ParseError::UnexpectedEof)?;
                        let value = value.swap_bytes() >> (64 - 24);
                        sign_extend(value, 24) as i32
                    }
                    3.. => {
                        let value = data.read_i32().ok_or(ParseError::UnexpectedEof)?;
                        i32::from_be(value as i32)
                    }
                }
            }
        }
    }

    Ok(result)
}

#[inline(always)]
/// Read `bits` bits using the manual API and sign extend into an i32.
// Ensure there is enough data available before calling.
fn next_as_i32(data: &mut Reader, bits: u32) -> i32 {
    let x = data.peek(bits);
    data.consume(bits);
    sign_extend(x, bits) as i32
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter;

    fn bytes(first: u8, zeros: usize) -> Vec<u8> {
        iter::once(first)
            .chain(iter::repeat(0).take(zeros))
            .collect()
    }

    #[test]
    fn all_02_bits() {
        let b = bytes(0x00, 0);
        let mut b = Reader::new(b.as_slice());

        assert_eq!([0; 3], tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_04_bits() {
        let b = bytes(0x40, 1);
        let mut b = Reader::new(b.as_slice());

        assert_eq!([0; 3], tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_06_bits() {
        let b = bytes(0x80, 2);
        let mut b = Reader::new(b.as_slice());

        assert_eq!([0; 3], tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_08_bits() {
        let b = bytes(0b1100_0000, 3);
        let mut b = Reader::new(b.as_slice());

        assert_eq!([0; 3], tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_16_bits() {
        let b = bytes(0b1101_0101, 6);
        let mut b = Reader::new(b.as_slice());

        assert_eq!([0; 3], tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_24_bits() {
        let b = bytes(0b1110_1010, 9);
        let mut b = Reader::new(b.as_slice());

        assert_eq!([0; 3], tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_32_bits() {
        let b = bytes(0xFF, 12);
        let mut b = Reader::new(b.as_slice());

        assert_eq!([0; 3], tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    #[should_panic(expected = "UnexpectedEof")]
    fn eof_04_bit() {
        let mut b = Reader::new(&[0x40]);
        tagged_32(&mut b).unwrap();
    }

    #[test]
    #[should_panic(expected = "UnexpectedEof")]
    fn eof_06_bit() {
        let mut b = Reader::new(&[0x80]);
        tagged_32(&mut b).unwrap();
    }
}
