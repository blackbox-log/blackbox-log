use super::sign_extend;
use crate::{ParseError, ParseResult, Reader};
use bitter::BitReader;
use std::array;
use tracing::instrument;

#[instrument(level = "trace", skip(data), ret)]
pub fn read_tagged_32(data: &mut Reader) -> ParseResult<[i32; 3]> {
    const COUNT: usize = 3;
    let mut result = [0; COUNT];

    if !data.has_bits_remaining(8) {
        return Err(ParseError::unexpected_eof());
    }

    match data.read_bits(2).unwrap() {
        0 => {
            for x in &mut result {
                *x = next_as_i32(data, 2)?;
            }
        }

        1 => {
            // Skip rest of tag byte
            data.read_bits(2);

            for x in &mut result {
                *x = next_as_i32(data, 4)?;
            }
        }

        2 => {
            result[0] = next_as_i32(data, 6)?;

            for x in result.iter_mut().skip(1) {
                // Skip upper 2 bits
                data.read_bits(2);
                *x = next_as_i32(data, 6)?;
            }
        }

        3 => {
            let tags: [_; 3] = array::from_fn(|_| data.read_bits(2).unwrap());
            for (x, tag) in result.iter_mut().zip(tags.iter().rev()) {
                match *tag {
                    0 => {
                        *x = next_as_i32(data, 8)?;
                    }
                    1 => {
                        let value = data.read_i16().ok_or_else(ParseError::unexpected_eof)?;
                        *x = i16::from_be(value).into();
                    }

                    2 => {
                        let value: u64 =
                            data.read_bits(24).ok_or_else(ParseError::unexpected_eof)?;
                        let value = value.swap_bytes() >> (64 - 24);
                        *x = sign_extend(value, 24) as i32;
                    }
                    3 => {
                        let value = data.read_i32().ok_or_else(ParseError::unexpected_eof)?;
                        *x = i32::from_be(value as i32);
                    }
                    4.. => unreachable!(),
                }
            }
        }
        4.. => unreachable!(),
    }

    Ok(result)
}

fn next_as_i32(data: &mut Reader, bits: u32) -> ParseResult<i32> {
    data.read_bits(bits)
        .map(|x| sign_extend(x, bits) as i32)
        .ok_or_else(ParseError::unexpected_eof)
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

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_04_bits() {
        let b = bytes(0x40, 1);
        let mut b = Reader::new(b.as_slice());

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_06_bits() {
        let b = bytes(0x80, 2);
        let mut b = Reader::new(b.as_slice());

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_08_bits() {
        let b = bytes(0b1100_0000, 3);
        let mut b = Reader::new(b.as_slice());

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_16_bits() {
        let b = bytes(0b1101_0101, 6);
        let mut b = Reader::new(b.as_slice());

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_24_bits() {
        let b = bytes(0b1110_1010, 9);
        let mut b = Reader::new(b.as_slice());

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_32_bits() {
        let b = bytes(0xFF, 12);
        let mut b = Reader::new(b.as_slice());

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }
}
