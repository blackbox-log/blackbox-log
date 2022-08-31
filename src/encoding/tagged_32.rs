use super::sign_extend;
use crate::{ParseError, ParseResult};
use biterator::{Biterator, CustomInt};
use std::io::Read;
use std::{array, cmp, iter, marker, ops};
use tracing::instrument;

#[instrument(level = "trace", skip(data), ret)]
pub fn read_tagged_32<R: Read>(data: &mut Biterator<R>) -> ParseResult<[i32; 3]> {
    const COUNT: usize = 3;
    let mut result = [0; COUNT];

    data.byte_align();

    // Ensure unwrap will succeed when getting tags
    data.peek_byte().ok_or_else(ParseError::unexpected_eof)?;

    match data.next_bits::<2>().unwrap().get() {
        0 => {
            for x in result.iter_mut() {
                *x = next_as_i32::<2, R>(data)?;
            }
        }

        1 => {
            // Skip rest of tag byte
            data.next_bits::<2>();

            for x in result.iter_mut() {
                *x = next_as_i32::<4, R>(data)?;
            }
        }

        2 => {
            result[0] = next_as_i32::<6, R>(data)?;

            for x in result.iter_mut().skip(1) {
                // Skip upper 2 bits
                data.next_bits::<2>();
                *x = next_as_i32::<6, R>(data)?;
            }
        }

        3 => {
            let mut tags: [_; 3] = array::from_fn(|_| data.next_bits::<2>().unwrap().get());
            for (x, tag) in result.iter_mut().zip(tags.iter().rev()) {
                match tag {
                    0 => {
                        *x = next_as_i32::<8, R>(data)?;
                    }
                    1 => {
                        let low = data.next_byte().ok_or_else(ParseError::unexpected_eof)?;
                        let high = data.next_byte().ok_or_else(ParseError::unexpected_eof)?;

                        *x = i16::from_be_bytes([high, low]).into();
                    }
                    2 => {
                        let mut bytes = 0;
                        for _ in 0..3 {
                            bytes |=
                                data.next_byte().ok_or_else(ParseError::unexpected_eof)? as u32;
                            bytes <<= 8;
                        }

                        bytes = bytes.swap_bytes();
                        *x = sign_extend(CustomInt::<u32, 24>::new(bytes));
                    }
                    3 => {
                        for _ in 0..4 {
                            *x <<= 8;
                            *x |= data.next_byte().ok_or_else(ParseError::unexpected_eof)? as i32;
                        }
                        *x = x.swap_bytes();
                    }
                    _ => unreachable!(),
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(result)
}

fn next_as_i32<const BITS: u8, R: Read>(data: &mut Biterator<R>) -> ParseResult<i32> {
    data.next_bits::<BITS>()
        .map(sign_extend)
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
        let mut b = Biterator::new(b.as_slice());

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.peek_bit().is_none());
    }

    #[test]
    fn all_04_bits() {
        let b = bytes(0x40, 1);
        let mut b = Biterator::new(b.as_slice());

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.peek_bit().is_none());
    }

    #[test]
    fn all_06_bits() {
        let b = bytes(0x80, 3);
        let mut b = Biterator::new(b.as_slice());

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.peek_bit().is_none());
    }

    #[test]
    fn all_08_bits() {
        let b = bytes(0b1100_0000, 3);
        let mut b = Biterator::new(b.as_slice());

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.peek_bit().is_none());
    }

    #[test]
    fn all_16_bits() {
        let b = bytes(0b1101_0101, 6);
        let mut b = Biterator::new(b.as_slice());

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.peek_bit().is_none());
    }

    #[test]
    fn all_24_bits() {
        let b = bytes(0b1110_1010, 9);
        let mut b = Biterator::new(b.as_slice());

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.peek_bit().is_none());
    }

    #[test]
    fn all_32_bits() {
        let b = bytes(0xFF, 12);
        let mut b = Biterator::new(b.as_slice());

        assert_eq!([0; 3], read_tagged_32(&mut b).unwrap());
        assert!(b.peek_bit().is_none());
    }
}
