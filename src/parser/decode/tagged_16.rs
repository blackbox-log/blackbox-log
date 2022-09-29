use bitter::BitReader;

use super::sign_extend;
use crate::parser::{ParseError, ParseResult, Reader};
use crate::LogVersion;

const COUNT: usize = 4;

pub fn tagged_16(version: LogVersion, data: &mut Reader) -> ParseResult<[i16; COUNT]> {
    data.byte_align();

    match version {
        LogVersion::V1 => tagged_16_v1(data),
        LogVersion::V2 => tagged_16_v2(data),
    }
}

fn tagged_16_v1(data: &mut Reader) -> ParseResult<[i16; 4]> {
    let mut bytes = data.bytes();

    let mut result = [0; COUNT];
    let tags = bytes.read_u8().ok_or(ParseError::UnexpectedEof)?;

    if tags == 0 {
        return Ok(result);
    }

    let mut i = 0;
    while i < COUNT {
        let tag = (tags >> (i * 2)) & 3;

        match tag {
            0 => result[i] = 0,
            1 => {
                // Avoid out-of-bounds error on second nibble
                if i == 3 {
                    return Err(ParseError::Corrupted);
                }

                let byte = bytes.read_u8().ok_or(ParseError::UnexpectedEof)?;

                // Lower nibble first...
                result[i] = i4_to_i16(byte & 0xF);
                i += 1;
                result[i] = i4_to_i16(byte >> 4);
            }
            2 => result[i] = bytes.read_i8().ok_or(ParseError::UnexpectedEof)?.into(),
            3.. => result[i] = bytes.read_i16().ok_or(ParseError::UnexpectedEof)?,
        }

        i += 1;
    }

    Ok(result)
}

fn tagged_16_v2(data: &mut Reader) -> ParseResult<[i16; 4]> {
    let bits = data.bits();

    let mut result = [0; COUNT];
    let tags = bits.read_u8().ok_or(ParseError::UnexpectedEof)?;

    if tags == 0 {
        return Ok(result);
    }

    let mut i = 0;
    while i < COUNT {
        let tag = (tags >> (i * 2)) & 3;

        match tag {
            0 => result[i] = 0,
            1 => {
                let nibble = bits.read_bits(4).ok_or(ParseError::UnexpectedEof)?;
                result[i] = i4_to_i16(nibble as u8);
            }
            2 => {
                let byte = bits.read_u8().ok_or(ParseError::UnexpectedEof)?;
                result[i] = (byte as i8).into();
            }
            3.. => {
                let bytes = bits.read_i16().ok_or(ParseError::UnexpectedEof)?;
                result[i] = bytes;
            }
        }

        i += 1;
    }

    Ok(result)
}

fn i4_to_i16(nibble: u8) -> i16 {
    sign_extend::<4>(nibble.into()) as i16
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::LogVersion::{V1, V2};
    use alloc::vec::Vec;
    use core::iter;
    use test_case::case;

    fn bytes(first: u8, zeros: usize) -> Vec<u8> {
        iter::once(first)
            .chain(iter::repeat(0).take(zeros))
            .collect()
    }

    #[case(V1 ; "v1")]
    #[case(V2 ; "v2")]
    fn all_zeros(version: LogVersion) {
        let bytes = bytes(0x00, 0);
        let bytes = bytes.as_slice();

        assert_eq!(
            [0; 4],
            super::tagged_16(version, &mut Reader::new(bytes)).unwrap()
        );
    }

    #[case(V1 ; "v1")]
    #[case(V2 ; "v2")]
    fn all_nibbles(version: LogVersion) {
        let bytes = bytes(0x55, 2);
        let bytes = bytes.as_slice();

        assert_eq!(
            [0; 4],
            super::tagged_16(version, &mut Reader::new(bytes)).unwrap()
        );
    }

    #[case(V1 ; "v1")]
    #[case(V2 ; "v2")]
    fn all_bytes(version: LogVersion) {
        let bytes = bytes(0xAA, 4);
        let bytes = bytes.as_slice();

        assert_eq!(
            [0; 4],
            super::tagged_16(version, &mut Reader::new(bytes)).unwrap()
        );
    }

    #[test]
    fn all_16_bits_v1() {
        let bytes: &[u8] = &[0xFF, 1, 0, 2, 0, 3, 0, 4, 0];
        let mut bits = Reader::new(bytes);

        let expected = [1, 2, 3, 4];
        assert_eq!(expected, super::tagged_16(V1, &mut bits).unwrap());
    }

    #[test]
    fn all_16_bits_v2() {
        let bytes: &[u8] = &[0xFF, 0, 1, 0, 2, 0, 3, 0, 4];
        let mut bits = Reader::new(bytes);

        let expected = [1, 2, 3, 4];
        assert_eq!(expected, super::tagged_16(V2, &mut bits).unwrap());
    }

    #[test]
    fn tag_order_v1() {
        let bytes: &[u8] = &[0b1001_0100, 0x21, 0x03];
        let mut bits = Reader::new(bytes);

        assert_eq!([0, 1, 2, 3], super::tagged_16(V1, &mut bits).unwrap());
    }

    #[test]
    fn tag_order_v2() {
        let bytes: &[u8] = &[0b1110_0100, 0x10, 0x20, 0x00, 0x30];
        let mut bits = Reader::new(bytes);

        assert_eq!([0, 1, 2, 3], super::tagged_16(V2, &mut bits).unwrap());
    }

    #[test]
    fn v1_nibble_ignores_next_tag() {
        let bytes: &[u8] = &[0b0000_1101, 0x21];
        let mut bits = Reader::new(bytes);

        let expected = [1, 2, 0x00, 0x00];
        assert_eq!(expected, super::tagged_16(V1, &mut bits).unwrap());
    }

    #[case(V1, &[1, 194] => [2, -4, 0, 0] ; "v1 low nibble first")]
    #[case(V1, &[10, 163, 10] => [-93, 10, 0, 0] ; "v1 8 bit sign extend")]
    #[case(V2, &[0x30, 181, 61] => [0, 0, -19139, 0] ; "v2 16 bit high byte first")]
    fn regressions(version: LogVersion, bytes: &[u8]) -> [i16; 4] {
        let mut bits = Reader::new(bytes);
        super::tagged_16(version, &mut bits).unwrap()
    }
}
