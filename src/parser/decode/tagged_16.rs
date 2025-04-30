use super::sign_extend;
use crate::parser::{InternalError, InternalResult};
use crate::utils::{as_i16, as_i8, as_u32};
use crate::Reader;

const COUNT: usize = 4;

pub(crate) fn tagged_16(data: &mut Reader, out: &mut [u32]) -> InternalResult<()> {
    assert_eq!(out.len(), COUNT);

    let tags = data.read_u8().ok_or(InternalError::Eof)?;

    if tags == 0 {
        out.fill(0);
        return Ok(());
    }

    let mut aligned = true;
    let mut buffer = 0;

    for (i, out) in out.iter_mut().enumerate() {
        *out = match (tags >> (i * 2)) & 3 {
            0 => 0,
            1 => {
                let nibble = if aligned {
                    buffer = data.read_u8().ok_or(InternalError::Eof)?;
                    buffer >> 4
                } else {
                    buffer & 0xF
                };

                aligned = !aligned;
                as_u32(i4_to_i16(nibble).into())
            }
            2 => {
                let byte = if aligned {
                    data.read_i8().ok_or(InternalError::Eof)?
                } else {
                    let upper = buffer << 4;
                    buffer = data.read_u8().ok_or(InternalError::Eof)?;
                    as_i8(upper | buffer >> 4)
                };

                as_u32(byte.into())
            }
            3.. => {
                if aligned {
                    let raw = data.read_i16().ok_or(InternalError::Eof)?.swap_bytes();
                    as_u32(raw.into())
                } else {
                    let upper = u16::from(buffer) << 12;
                    let [middle, lower] = data.read_u16().ok_or(InternalError::Eof)?.to_le_bytes();

                    buffer = lower;
                    as_u32(as_i16(upper | (u16::from(middle) << 4) | u16::from(lower >> 4)).into())
                }
            }
        }
    }

    Ok(())
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
    use crate::utils::as_i32;

    fn bytes(first: u8, zeros: usize) -> Vec<u8> {
        iter::once(first)
            .chain(iter::repeat(0).take(zeros))
            .collect()
    }

    #[test]
    fn all_zeros() {
        let bytes = bytes(0x00, 0);

        let mut out = [0; 4];
        tagged_16(&mut Reader::new(&bytes), &mut out).unwrap();
        assert_eq!([0; 4], out);
    }

    #[test]
    fn all_nibbles() {
        let bytes = bytes(0x55, 2);

        let mut out = [0; 4];
        tagged_16(&mut Reader::new(&bytes), &mut out).unwrap();
        assert_eq!([0; 4], out);
    }

    #[test]
    fn all_bytes() {
        let bytes = bytes(0xAA, 4);

        let mut out = [0; 4];
        tagged_16(&mut Reader::new(&bytes), &mut out).unwrap();
        assert_eq!([0; 4], out);
    }

    #[test]
    fn all_16_bits() {
        let bytes = [0xFF, 0, 1, 0, 2, 0, 3, 0, 4];

        let mut out = [0; 4];
        tagged_16(&mut Reader::new(&bytes), &mut out).unwrap();
        assert_eq!([1, 2, 3, 4], out);
    }

    #[test]
    fn tag_order_v2() {
        let bytes = [0b1110_0100, 0x10, 0x20, 0x00, 0x30];

        let mut out = [0; 4];
        tagged_16(&mut Reader::new(&bytes), &mut out).unwrap();
        assert_eq!([0, 1, 2, 3], out);
    }

    #[case(&[0x30, 181, 61] => [0, 0, -19139, 0] ; "16 bit high byte first")]
    fn regressions(bytes: &[u8]) -> [i32; 4] {
        let mut out = [0; 4];
        tagged_16(&mut Reader::new(bytes), &mut out).unwrap();
        out.map(as_i32)
    }
}
