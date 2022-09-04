use super::sign_extend;
use crate::{LogVersion, ParseError, ParseResult, Reader};
use bitter::BitReader;
use tracing::instrument;

#[instrument(level = "trace", skip(data), ret)]
pub fn read_tagged_16(version: LogVersion, data: &mut Reader) -> ParseResult<[i16; 4]> {
    const COUNT: usize = 4;

    // FIXME: data.byte_align();

    let tags = data.read_u8().ok_or_else(ParseError::unexpected_eof)?;
    let mut result = [0; COUNT];

    let mut i = 0;
    while i < COUNT {
        let tag = (tags >> (i * 2)) & 3;

        match tag {
            0 => result[i] = 0,
            1 => match version {
                LogVersion::V1 => {
                    // Avoid OOB error on second nibble
                    if i == 3 {
                        return Err(ParseError::Corrupted);
                    }

                    let byte = data.read_u8().ok_or_else(ParseError::unexpected_eof)?;

                    // Lower nibble first...
                    result[i] = i4_to_i16(byte & 0xF);
                    i += 1;
                    result[i] = i4_to_i16(byte >> 4);
                }
                LogVersion::V2 => {
                    let nibble = data.read_bits(4).ok_or_else(ParseError::unexpected_eof)?;
                    result[i] = i4_to_i16(nibble as u8);
                }
            },
            2 => {
                result[i] = (data.read_i8().ok_or_else(ParseError::unexpected_eof)?).into();
            }
            3 => {
                let bytes = data.read_i16().ok_or_else(ParseError::unexpected_eof)?;

                result[i] = match version {
                    LogVersion::V1 => i16::from_be(bytes),
                    LogVersion::V2 => i16::from_le(bytes),
                };
            }
            4.. => unreachable!(),
        }

        i += 1;
    }

    Ok(result)
}

fn i4_to_i16(nibble: u8) -> i16 {
    sign_extend(nibble.into(), 4) as i16
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::LogVersion::{V1, V2};
    use std::iter;
    use test_case::case;

    fn bytes(first: u8, zeros: usize) -> Vec<u8> {
        iter::once(first)
            .chain(iter::repeat(0).take(zeros))
            .collect()
    }

    #[test]
    fn all_zeros() {
        let bytes = bytes(0x00, 0);
        let bytes = bytes.as_slice();

        assert_eq!(
            [0; 4],
            super::read_tagged_16(V1, &mut Reader::new(bytes)).unwrap()
        );
        assert_eq!(
            [0; 4],
            super::read_tagged_16(V2, &mut Reader::new(bytes)).unwrap()
        );
    }

    #[test]
    fn all_nibbles() {
        let bytes = bytes(0x55, 2);
        let bytes = bytes.as_slice();

        assert_eq!(
            [0; 4],
            super::read_tagged_16(V1, &mut Reader::new(bytes)).unwrap()
        );
        assert_eq!(
            [0; 4],
            super::read_tagged_16(V2, &mut Reader::new(bytes)).unwrap()
        );
    }

    #[test]
    fn all_bytes() {
        let bytes = bytes(0xAA, 4);
        let bytes = bytes.as_slice();

        assert_eq!(
            [0; 4],
            super::read_tagged_16(V1, &mut Reader::new(bytes)).unwrap()
        );
        assert_eq!(
            [0; 4],
            super::read_tagged_16(V2, &mut Reader::new(bytes)).unwrap()
        );
    }

    #[test]
    fn all_16_bits_v1() {
        let bytes: &[u8] = &[0xFF, 1, 0, 2, 0, 3, 0, 4, 0];
        let mut bits = Reader::new(bytes);

        let expected = [1, 2, 3, 4];
        assert_eq!(expected, super::read_tagged_16(V1, &mut bits).unwrap());
    }

    #[test]
    fn all_16_bits_v2() {
        let bytes: &[u8] = &[0xFF, 0, 1, 0, 2, 0, 3, 0, 4];
        let mut bits = Reader::new(bytes);

        let expected = [1, 2, 3, 4];
        assert_eq!(expected, super::read_tagged_16(V2, &mut bits).unwrap());
    }

    #[test]
    fn tag_order_v1() {
        let bytes: &[u8] = &[0b1001_0100, 0x21, 0x03];
        let mut bits = Reader::new(bytes);

        assert_eq!([0, 1, 2, 3], super::read_tagged_16(V1, &mut bits).unwrap());
    }

    #[test]
    fn tag_order_v2() {
        let bytes: &[u8] = &[0b1110_0100, 0x10, 0x20, 0x00, 0x30];
        let mut bits = Reader::new(bytes);

        assert_eq!([0, 1, 2, 3], super::read_tagged_16(V2, &mut bits).unwrap());
    }

    #[test]
    fn v1_nibble_ignores_next_tag() {
        let bytes: &[u8] = &[0b0000_1101, 0x21];
        let mut bits = Reader::new(bytes);

        let expected = [1, 2, 0x00, 0x00];
        assert_eq!(expected, super::read_tagged_16(V1, &mut bits).unwrap());
    }

    #[case(V1, &[1, 194] => [2, -4, 0, 0] ; "low nibble first")]
    #[case(V1, &[10, 163, 10] => [-93, 10, 0, 0] ; "8 bit sign extend")]
    fn diff_from_reference(version: LogVersion, bytes: &[u8]) -> [i16; 4] {
        let mut bits = Reader::new(bytes);
        super::read_tagged_16(version, &mut bits).unwrap()
    }
}
