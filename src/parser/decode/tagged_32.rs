use super::sign_extend;
use crate::parser::{reader::ByteReader, ParseError, ParseResult, Reader};

const COUNT: usize = 3;

#[allow(clippy::assertions_on_constants)]
pub fn tagged_32(data: &mut Reader) -> ParseResult<[i32; COUNT]> {
    fn read_u8_or_eof(bytes: &mut ByteReader) -> ParseResult<u8> {
        bytes.read_u8().ok_or(ParseError::UnexpectedEof)
    }

    let mut bytes = data.bytes();

    let mut result = [0; COUNT];

    let byte = read_u8_or_eof(&mut bytes)?;
    match (byte & 0xC0) >> 6 {
        // 2 bits
        0 => {
            #[inline(always)]
            fn convert(x: u8) -> i32 {
                sign_extend::<2>((x & 3).into())
            }

            result[0] = convert(byte >> 4);
            result[1] = convert(byte >> 2);
            result[2] = convert(byte);
        }

        // 4 bits
        1 => {
            #[inline(always)]
            fn convert(x: u8) -> i32 {
                sign_extend::<4>(x.into())
            }

            result[0] = convert(byte & 0x0F);

            let byte = read_u8_or_eof(&mut bytes)?;
            result[1] = convert(byte >> 4);
            result[2] = convert(byte & 0x0F);
        }

        // 6 bits
        2 => {
            #[inline(always)]
            fn convert(x: u8) -> i32 {
                sign_extend::<6>((x & 0x3F).into())
            }

            result[0] = convert(byte);

            let byte = read_u8_or_eof(&mut bytes)?;
            result[1] = convert(byte);

            let byte = read_u8_or_eof(&mut bytes)?;
            result[2] = convert(byte);
        }

        3.. => {
            let mut tags = byte & 0x3F;
            for x in &mut result {
                let tag = tags & 3;
                tags >>= 2;

                *x = match tag {
                    // 8 bits
                    0 => {
                        let x = read_u8_or_eof(&mut bytes)?;
                        sign_extend::<8>(x.into()) as i32
                    }

                    // 16 bits
                    1 => {
                        let value = bytes.read_u16().ok_or(ParseError::UnexpectedEof)?;
                        (value as i16).into()
                    }

                    // 24 bits
                    2 => {
                        let x = bytes.read_u24().ok_or(ParseError::UnexpectedEof)?;
                        sign_extend::<24>(x)
                    }

                    // 32 bits
                    3.. => {
                        let value = bytes.read_u32().ok_or(ParseError::UnexpectedEof)?;
                        value as i32
                    }
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

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

        assert_eq!([0, -1, 1], tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_04_bits() {
        let b = [0x41, 0x23];
        let mut b = Reader::new(&b);

        assert_eq!([1, 2, 3], tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_06_bits() {
        let b = [0x81, 0x02, 0x03];
        let mut b = Reader::new(&b);

        assert_eq!([1, 2, 3], tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_08_bits() {
        let b = bytes(0, 1);
        let mut b = Reader::new(&b);

        assert_eq!([1, 2, 3], tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_16_bits() {
        let b = bytes(1, 2);
        let mut b = Reader::new(&b);

        assert_eq!([1, 2, 3], tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_24_bits() {
        let b = bytes(2, 3);
        let mut b = Reader::new(&b);

        assert_eq!([1, 2, 3], tagged_32(&mut b).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn all_32_bits() {
        let b = bytes(3, 4);
        let mut b = Reader::new(&b);

        assert_eq!([1, 2, 3], tagged_32(&mut b).unwrap());
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
