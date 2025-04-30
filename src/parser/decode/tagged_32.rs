use super::sign_extend;
use crate::parser::{InternalError, InternalResult};
use crate::utils::{as_i16, as_i8, as_u32};
use crate::Reader;

const COUNT: usize = 3;

pub(crate) fn tagged_32(data: &mut Reader, out: &mut [u32]) -> InternalResult<()> {
    fn read_u8_or_eof(bytes: &mut Reader) -> InternalResult<u8> {
        bytes.read_u8().ok_or(InternalError::Eof)
    }

    assert_eq!(out.len(), COUNT);

    let byte = read_u8_or_eof(data)?;
    match (byte & 0xC0) >> 6 {
        // 2 bits
        0 => {
            #[inline(always)]
            fn convert(x: u8) -> u32 {
                as_u32(sign_extend::<2>((x & 3).into()))
            }

            out[0] = convert(byte >> 4);
            out[1] = convert(byte >> 2);
            out[2] = convert(byte);
        }

        // 4 bits
        1 => {
            #[inline(always)]
            fn convert(x: u8) -> u32 {
                as_u32(sign_extend::<4>(x.into()))
            }

            out[0] = convert(byte & 0x0F);

            let byte = read_u8_or_eof(data)?;
            out[1] = convert(byte >> 4);
            out[2] = convert(byte & 0x0F);
        }

        // 6 bits
        2 => {
            #[inline(always)]
            fn convert(x: u8) -> u32 {
                as_u32(sign_extend::<6>((x & 0x3F).into()))
            }

            out[0] = convert(byte);

            let byte = read_u8_or_eof(data)?;
            out[1] = convert(byte);

            let byte = read_u8_or_eof(data)?;
            out[2] = convert(byte);
        }

        3.. => {
            let mut tags = byte & 0x3F;
            for x in out {
                let tag = tags & 3;
                tags >>= 2;

                *x = match tag {
                    // 8 bits
                    0 => {
                        let x = read_u8_or_eof(data)?;
                        as_u32(as_i8(x).into())
                    }

                    // 16 bits
                    1 => {
                        let value = data.read_u16().ok_or(InternalError::Eof)?;
                        as_u32(as_i16(value).into())
                    }

                    // 24 bits
                    2 => {
                        let x = data.read_u24().ok_or(InternalError::Eof)?;
                        as_u32(sign_extend::<24>(x))
                    }

                    // 32 bits
                    3.. => data.read_u32().ok_or(InternalError::Eof)?,
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;
    use crate::utils::as_i32;

    fn bytes(tag: u8, width: usize) -> Vec<u8> {
        assert_eq!(tag, tag & 3);

        let tag = 0xC0 | (tag << 4) | (tag << 2) | tag;
        let mut bytes = vec![tag];

        for i in 1..=3 {
            bytes.push(i);
            bytes.append(&mut vec![0; width - 1]);
        }

        bytes
    }

    #[test]
    fn all_02_bits() {
        let b = [0x0D];
        let mut b = Reader::new(&b);

        let mut out = [0; 3];
        tagged_32(&mut b, &mut out).unwrap();
        assert_eq!([0, -1, 1], out.map(as_i32));
        assert!(b.is_empty());
    }

    #[test]
    fn all_04_bits() {
        let b = [0x41, 0x23];
        let mut b = Reader::new(&b);

        let mut out = [0; 3];
        tagged_32(&mut b, &mut out).unwrap();
        assert_eq!([1, 2, 3], out.map(as_i32));
        assert!(b.is_empty());
    }

    #[test]
    fn all_06_bits() {
        let b = [0x81, 0x02, 0x03];
        let mut b = Reader::new(&b);

        let mut out = [0; 3];
        tagged_32(&mut b, &mut out).unwrap();
        assert_eq!([1, 2, 3], out.map(as_i32));
        assert!(b.is_empty());
    }

    #[test]
    fn all_08_bits() {
        let b = bytes(0, 1);
        let mut b = Reader::new(&b);

        let mut out = [0; 3];
        tagged_32(&mut b, &mut out).unwrap();
        assert_eq!([1, 2, 3], out.map(as_i32));
        assert!(b.is_empty());
    }

    #[test]
    fn all_16_bits() {
        let b = bytes(1, 2);
        let mut b = Reader::new(&b);

        let mut out = [0; 3];
        tagged_32(&mut b, &mut out).unwrap();
        assert_eq!([1, 2, 3], out.map(as_i32));
        assert!(b.is_empty());
    }

    #[test]
    fn all_24_bits() {
        let b = bytes(2, 3);
        let mut b = Reader::new(&b);

        let mut out = [0; 3];
        tagged_32(&mut b, &mut out).unwrap();
        assert_eq!([1, 2, 3], out.map(as_i32));
        assert!(b.is_empty());
    }

    #[test]
    fn all_32_bits() {
        let b = bytes(3, 4);
        let mut b = Reader::new(&b);

        let mut out = [0; 3];
        tagged_32(&mut b, &mut out).unwrap();
        assert_eq!([1, 2, 3], out.map(as_i32));
        assert!(b.is_empty());
    }

    #[test]
    #[should_panic(expected = "Eof")]
    fn eof_04_bit() {
        let mut b = Reader::new(&[0x40]);
        tagged_32(&mut b, &mut [0; 3]).unwrap();
    }

    #[test]
    #[should_panic(expected = "Eof")]
    fn eof_06_bit() {
        let mut b = Reader::new(&[0x80]);
        tagged_32(&mut b, &mut [0; 3]).unwrap();
    }
}
