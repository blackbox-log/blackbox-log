use crate::{LogVersion, ParseError, ParseResult};
use biterator::Biterator;
use std::io::Read;
use tracing::instrument;

#[instrument(level = "trace", skip(data), ret)]
pub fn read_tagged_16<R: Read>(
    version: LogVersion,
    data: &mut Biterator<R>,
) -> ParseResult<[i16; 4]> {
    fn i4_to_i16(i4: u8) -> i16 {
        let i4 = u16::from(i4);
        let byte = if (i4 & 8) > 0 { i4 | 0xFFF0 } else { i4 };
        byte as i16
    }

    const COUNT: usize = 4;

    data.byte_align();

    let tags = data.next_byte().ok_or_else(ParseError::unexpected_eof)?;
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

                    let byte = data.next_byte().ok_or_else(ParseError::unexpected_eof)?;

                    // Lower nibble first...
                    result[i] = i4_to_i16(byte & 0xF);
                    i += 1;
                    result[i] = i4_to_i16(byte >> 4);
                }
                LogVersion::V2 => {
                    result[i] = i4_to_i16(
                        data.next_nibble()
                            .ok_or_else(ParseError::unexpected_eof)?
                            .get(),
                    );
                }
            },
            2 => {
                result[i] = (data.next_byte().ok_or_else(ParseError::unexpected_eof)? as i8).into();
            }
            3 => {
                let byte1 = data.next_byte().ok_or_else(ParseError::unexpected_eof)?;
                let byte2 = data.next_byte().ok_or_else(ParseError::unexpected_eof)?;
                let bytes = [byte1, byte2];

                result[i] = match version {
                    LogVersion::V1 => i16::from_le_bytes(bytes),
                    LogVersion::V2 => i16::from_be_bytes(bytes),
                };
            }
            4.. => unreachable!(),
        }

        i += 1;
    }

    Ok(result)
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
            super::read_tagged_16(V1, &mut Biterator::new(bytes)).unwrap()
        );
        assert_eq!(
            [0; 4],
            super::read_tagged_16(V2, &mut Biterator::new(bytes)).unwrap()
        );
    }

    #[test]
    fn all_nibbles() {
        let bytes = bytes(0x55, 2);
        let bytes = bytes.as_slice();

        assert_eq!(
            [0; 4],
            super::read_tagged_16(V1, &mut Biterator::new(bytes)).unwrap()
        );
        assert_eq!(
            [0; 4],
            super::read_tagged_16(V2, &mut Biterator::new(bytes)).unwrap()
        );
    }

    #[test]
    fn all_bytes() {
        let bytes = bytes(0xAA, 4);
        let bytes = bytes.as_slice();

        assert_eq!(
            [0; 4],
            super::read_tagged_16(V1, &mut Biterator::new(bytes)).unwrap()
        );
        assert_eq!(
            [0; 4],
            super::read_tagged_16(V2, &mut Biterator::new(bytes)).unwrap()
        );
    }

    #[test]
    fn all_16_bits_v1() {
        let bytes: &[u8] = &[0xFF, 1, 0, 2, 0, 3, 0, 4, 0];
        let mut biterator = Biterator::new(bytes);

        let expected = [1, 2, 3, 4];
        assert_eq!(expected, super::read_tagged_16(V1, &mut biterator).unwrap());
    }

    #[test]
    fn all_16_bits_v2() {
        let bytes: &[u8] = &[0xFF, 0, 1, 0, 2, 0, 3, 0, 4];
        let mut biterator = Biterator::new(bytes);

        let expected = [1, 2, 3, 4];
        assert_eq!(expected, super::read_tagged_16(V2, &mut biterator).unwrap());
    }

    #[test]
    fn tag_order_v1() {
        let bytes: &[u8] = &[0b1001_0100, 0x21, 0x03];
        let mut biterator = Biterator::new(bytes);

        assert_eq!(
            [0, 1, 2, 3],
            super::read_tagged_16(V1, &mut biterator).unwrap()
        );
    }

    #[test]
    fn tag_order_v2() {
        let bytes: &[u8] = &[0b1110_0100, 0x10, 0x20, 0x00, 0x30];
        let mut biterator = Biterator::new(bytes);

        assert_eq!(
            [0, 1, 2, 3],
            super::read_tagged_16(V2, &mut biterator).unwrap()
        );
    }

    #[test]
    fn v1_nibble_ignores_next_tag() {
        let bytes: &[u8] = &[0b0000_1101, 0x21];
        let mut biterator = Biterator::new(bytes);

        let expected = [1, 2, 0x00, 0x00];
        assert_eq!(expected, super::read_tagged_16(V1, &mut biterator).unwrap());
    }

    #[case(V1, &[1, 194] => [2, -4, 0, 0] ; "low nibble first")]
    #[case(V1, &[10, 163, 10] => [-93, 10, 0, 0] ; "8 bit sign extend")]
    fn diff_from_reference(version: LogVersion, bytes: &[u8]) -> [i16; 4] {
        let mut biterator = Biterator::new(bytes);
        super::read_tagged_16(version, &mut biterator).unwrap()
    }
}
