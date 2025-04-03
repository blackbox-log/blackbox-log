use super::sign_extend;
use crate::parser::{InternalError, InternalResult};
use crate::utils::{as_i16, as_i8};
use crate::Reader;

const COUNT: usize = 4;

pub(crate) fn tagged_16(data: &mut Reader) -> InternalResult<[i16; COUNT]> {
    let tags = data.read_u8().ok_or(InternalError::Eof)?;

    if tags == 0 {
        return Ok([0; COUNT]);
    }

    let mut result = [0; COUNT];
    let mut aligned = true;
    let mut buffer = 0;

    for (i, result) in result.iter_mut().enumerate() {
        *result = match (tags >> (i * 2)) & 3 {
            0 => 0,
            1 => {
                let nibble = if aligned {
                    buffer = data.read_u8().ok_or(InternalError::Eof)?;
                    buffer >> 4
                } else {
                    buffer & 0xF
                };

                aligned = !aligned;
                i4_to_i16(nibble)
            }
            2 => {
                let byte = if aligned {
                    data.read_i8().ok_or(InternalError::Eof)?
                } else {
                    let upper = buffer << 4;
                    buffer = data.read_u8().ok_or(InternalError::Eof)?;
                    as_i8(upper | buffer >> 4)
                };

                byte.into()
            }
            3.. => {
                if aligned {
                    data.read_i16().ok_or(InternalError::Eof)?.swap_bytes()
                } else {
                    let upper = u16::from(buffer) << 12;
                    let [middle, lower] = data.read_u16().ok_or(InternalError::Eof)?.to_le_bytes();

                    buffer = lower;
                    as_i16(upper | (u16::from(middle) << 4) | u16::from(lower >> 4))
                }
            }
        }
    }

    Ok(result)
}

#[inline]
fn i4_to_i16(nibble: u8) -> i16 {
    sign_extend::<4>(nibble.into()) as i16
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use core::iter;

    use test_case::case;

    use super::*;

    fn bytes(first: u8, zeros: usize) -> Vec<u8> {
        iter::once(first)
            .chain(iter::repeat(0).take(zeros))
            .collect()
    }

    #[test]
    fn all_zeros() {
        let bytes = bytes(0x00, 0);
        let bytes = bytes.as_slice();

        assert_eq!([0; 4], tagged_16(&mut Reader::new(bytes)).unwrap());
    }

    #[test]
    fn all_nibbles() {
        let bytes = bytes(0x55, 2);
        let bytes = bytes.as_slice();

        assert_eq!([0; 4], tagged_16(&mut Reader::new(bytes)).unwrap());
    }

    #[test]
    fn all_bytes() {
        let bytes = bytes(0xAA, 4);
        let bytes = bytes.as_slice();

        assert_eq!([0; 4], tagged_16(&mut Reader::new(bytes)).unwrap());
    }

    #[test]
    fn all_16_bits() {
        let bytes: &[u8] = &[0xFF, 0, 1, 0, 2, 0, 3, 0, 4];
        let mut bits = Reader::new(bytes);

        let expected = [1, 2, 3, 4];
        assert_eq!(expected, tagged_16(&mut bits).unwrap());
    }

    #[test]
    fn tag_order_v2() {
        let bytes: &[u8] = &[0b1110_0100, 0x10, 0x20, 0x00, 0x30];
        let mut bits = Reader::new(bytes);

        assert_eq!([0, 1, 2, 3], tagged_16(&mut bits).unwrap());
    }

    #[case( &[0x30, 181, 61] => [0, 0, -19139, 0] ; "16 bit high byte first")]
    fn regressions(bytes: &[u8]) -> [i16; 4] {
        let mut bits = Reader::new(bytes);
        tagged_16(&mut bits).unwrap()
    }
}
