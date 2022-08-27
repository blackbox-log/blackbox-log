use super::{LogVersion, ParseError, ParseResult};
use biterator::arbitrary_int::u1;
use biterator::Biterator;
use num_enum::TryFromPrimitive;
use std::io::Read;
use tracing::instrument;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum Encoding {
    /// Signed variable byte
    IVar = 0,
    /// Unsigned variable byte
    UVar,
    /// Unsigned variable byte, but negated after decoding. Value fits in 14 bits
    Negative14Bit,
    U32EliasDelta,
    I32EliasDelta,
    TaggedVar,
    Tagged32,
    /// 1 tag byte containing 4 2 bit tags, followed by 4 fields
    ///
    /// | Tag | Field width         |
    /// |-----|---------------------|
    /// | 0   | 0 (field value = 0) |
    /// | 1   | 4                   |
    /// | 2   | 8                   |
    /// | 3   | 16                  |
    Tagged16,
    /// Nothing is written to the log, assume value is 0
    Null,
    U32EliasGamma,
    I32EliasGamma,
}

#[instrument(level = "trace", skip(data), ret)]
pub fn read_uvar<R: Read>(data: &mut Biterator<R>) -> ParseResult<u32> {
    data.byte_align();

    let mut uvar: u32 = 0;
    for i in 0.. {
        let is_last_byte = match data.next_bit() {
            Some(bit) => bit == u1::new(0),
            None => return Err(ParseError::unexpected_eof()),
        };

        // Unwrap is safe after byte_align() above
        let byte = data.next_bits::<7>().unwrap();
        let byte = u32::from(byte)
            .checked_shl(7 * i)
            .ok_or(ParseError::Corrupted)?;
        uvar |= byte;

        if is_last_byte {
            break;
        }
    }

    Ok(uvar)
}

#[instrument(level = "trace", skip(data), ret)]
pub fn read_ivar<R: Read>(data: &mut Biterator<R>) -> ParseResult<i32> {
    read_uvar(data).map(zig_zag_decode)
}

#[instrument(level = "trace", skip(data), ret)]
pub fn read_negative_14_bit<R: Read>(data: &mut Biterator<R>) -> i32 {
    data.byte_align();

    unimplemented!();
}

#[instrument(level = "trace", skip(data), ret)]
pub fn read_u32_elias_delta<R: Read>(data: &mut Biterator<R>) -> ParseResult<u32> {
    let mut bits = data.bits();

    let mut leading_zeros: u8 = 0;
    for _ in 0..6 {
        match bits.next() {
            Some(bit) if bit == u1::new(0) => leading_zeros += 1,
            Some(_) => break,
            None => return Err(ParseError::unexpected_eof()),
        }
    }

    let leading_zeros = leading_zeros;
    if leading_zeros > 5 {
        return Err(ParseError::Corrupted);
    }

    let mut read = |count: u8| -> ParseResult<u32> {
        let mut result = 1;
        for i in 0..count {
            let bit = bits.next().ok_or_else(ParseError::unexpected_eof)?;
            result <<= 1;
            result += u32::from(bit);
        }
        Ok(result - 1)
    };

    // Guaranteed to be <= 31 since we're reading at most 5 bits
    let len = read(leading_zeros)? as u8;
    if len > 31 {
        return Err(ParseError::Corrupted);
    }

    let result = read(len)?;

    if result == (u32::MAX - 1) {
        // Use an extra bit to disambiguate (u32::MAX - 1) and u32::MAX
        let bit = bits.next().ok_or(ParseError::Corrupted)?;
        let bit: u32 = bit.into();
        Ok(result + bit)
    } else {
        Ok(result)
    }
}

#[instrument(level = "trace", skip(data), ret)]
pub fn read_i32_elias_delta<R: Read>(data: &mut Biterator<R>) -> ParseResult<i32> {
    read_u32_elias_delta(data).map(zig_zag_decode)
}

#[instrument(level = "trace", skip(data), ret)]
pub fn read_tagged_16<R: Read>(
    version: LogVersion,
    data: &mut Biterator<R>,
) -> ParseResult<[i16; 4]> {
    fn i4_to_i16(i4: u8) -> i16 {
        let i4 = i4 as u16;
        let byte = if (i4 & 8) > 0 { i4 ^ 0xFFF0 } else { i4 };
        byte as i16
    }

    data.byte_align();

    const COUNT: usize = 4;

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
                    result[i] = i4_to_i16(byte >> 4)
                }
                LogVersion::V2 => {
                    result[i] = i4_to_i16(
                        data.next_nibble()
                            .ok_or_else(ParseError::unexpected_eof)?
                            .value(),
                    )
                }
            },
            2 => {
                result[i] = (data.next_byte().ok_or_else(ParseError::unexpected_eof)? as i8).into()
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

const fn zig_zag_decode(value: u32) -> i32 {
    (value >> 1) as i32 ^ -(value as i32 & 1)
}

#[cfg(test)]
mod test {
    use crate::{LogVersion, ParseError, ParseResult};
    use biterator::Biterator;
    use std::iter;
    use test_case::case;
    use LogVersion::{V1, V2};

    #[test]
    fn read_uvar() {
        fn read(bytes: &[u8]) -> u32 {
            super::read_uvar(&mut Biterator::new(bytes)).unwrap()
        }

        assert_eq!(0, read(&[0x00]));
        assert_eq!(0, read(&[0x80, 0x00]));
        assert_eq!(1, read(&[1]));
        assert_eq!(0xFF, read(&[0xFF, 0x01]));
        assert_eq!(0x3FFF, read(&[0xFF, 0x7F]));
    }

    #[test]
    fn read_uvar_corrupted() {
        fn read(bytes: &[u8]) -> ParseResult<u32> {
            super::read_uvar(&mut Biterator::new(bytes))
        }

        let err = read(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        assert!(matches!(err, Err(ParseError::Corrupted)));
    }

    #[test]
    fn read_u32_elias_delta() {
        fn read(bytes: &[u8]) -> u32 {
            super::read_u32_elias_delta(&mut Biterator::new(bytes)).unwrap()
        }

        assert_eq!(0, read(&[0x80, 0]));
        assert_eq!(1, read(&[0x40, 0]));
        assert_eq!(2, read(&[0x50, 0]));
        assert_eq!(15, read(&[0x28, 0]));
        assert_eq!(18, read(&[0x29, 0x80]));

        assert_eq!(8191, read(&[0x1C, 0, 0]));
    }

    #[test]
    fn read_u32_elias_delta_max() {
        let bytes = &[0x04, 0x1F, 0xFF, 0xFF, 0xFF, 0xE0];
        let mut biterator = Biterator::new(bytes.as_slice());

        assert_eq!(
            u32::MAX,
            super::read_u32_elias_delta(&mut biterator).unwrap()
        );
    }

    #[test]
    fn read_i32_elias_delta() {
        fn read(bytes: &[u8]) -> i32 {
            super::read_i32_elias_delta(&mut Biterator::new(bytes)).unwrap()
        }

        assert_eq!(0, read(&[0x80, 0]));
        assert_eq!(-1, read(&[0x40, 0]));
        assert_eq!(1, read(&[0x50, 0]));
        assert_eq!(-8, read(&[0x28, 0]));
    }

    #[test]
    fn read_i32_elias_delta_min() {
        let bytes = &[0x04, 0x1F, 0xFF, 0xFF, 0xFF, 0xE0];
        let mut biterator = Biterator::new(bytes.as_slice());

        assert_eq!(
            i32::MIN,
            super::read_i32_elias_delta(&mut biterator).unwrap()
        );
    }

    #[test]
    fn read_i32_elias_delta_max() {
        let bytes = &[0x04, 0x1F, 0xFF, 0xFF, 0xFF, 0xC0];
        let mut biterator = Biterator::new(bytes.as_slice());

        assert_eq!(
            i32::MAX,
            super::read_i32_elias_delta(&mut biterator).unwrap()
        );
    }

    fn tagged_16_bytes(first: u8, zeros: usize) -> Vec<u8> {
        iter::once(first)
            .chain(iter::repeat(0).take(zeros))
            .collect()
    }

    #[test]
    fn tagged_16_all_zeros() {
        let bytes = tagged_16_bytes(0x00, 0);
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
    fn tagged_16_all_nibbles() {
        let bytes = tagged_16_bytes(0x55, 2);
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
    fn tagged_16_all_bytes() {
        let bytes = tagged_16_bytes(0xAA, 4);
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
    fn tagged_16_v1_all_16_bits() {
        let bytes: &[u8] = &[0xFF, 1, 0, 2, 0, 3, 0, 4, 0];
        let mut biterator = Biterator::new(bytes);

        let expected = [1, 2, 3, 4];
        assert_eq!(expected, super::read_tagged_16(V1, &mut biterator).unwrap());
    }

    #[test]
    fn tagged_16_v2_all_16_bits() {
        let bytes: &[u8] = &[0xFF, 0, 1, 0, 2, 0, 3, 0, 4];
        let mut biterator = Biterator::new(bytes);

        let expected = [1, 2, 3, 4];
        assert_eq!(expected, super::read_tagged_16(V2, &mut biterator).unwrap());
    }

    #[test]
    fn tagged_16_v1_tag_order() {
        let bytes: &[u8] = &[0b1001_0100, 0x21, 0x03];
        let mut biterator = Biterator::new(bytes);

        assert_eq!(
            [0, 1, 2, 3],
            super::read_tagged_16(V1, &mut biterator).unwrap()
        );
    }

    #[test]
    fn tagged_16_v2_tag_order() {
        let bytes: &[u8] = &[0b1110_0100, 0x10, 0x20, 0x00, 0x30];
        let mut biterator = Biterator::new(bytes);

        assert_eq!(
            [0, 1, 2, 3],
            super::read_tagged_16(V2, &mut biterator).unwrap()
        );
    }

    #[test]
    fn tagged_16_v1_nibble_ignores_next_tag() {
        let bytes: &[u8] = &[0b0000_1101, 0x21];
        let mut biterator = Biterator::new(bytes);

        let expected = [1, 2, 0x00, 0x00];
        assert_eq!(expected, super::read_tagged_16(V1, &mut biterator).unwrap());
    }

    #[case(V1, &[1, 194] => [2, -4, 0, 0] ; "low nibble first")]
    #[case(V1, &[10, 163, 10] => [-93, 10, 0, 0] ; "8 bit sign extend")]
    fn tagged_16_diff_from_reference(version: LogVersion, bytes: &[u8]) -> [i16; 4] {
        let mut biterator = Biterator::new(bytes);
        super::read_tagged_16(version, &mut biterator).unwrap()
    }

    #[test]
    fn zig_zag_decode() {
        use super::zig_zag_decode;

        assert_eq!(0, zig_zag_decode(0));
        assert_eq!(-1, zig_zag_decode(1));
        assert_eq!(1, zig_zag_decode(2));
        assert_eq!(-2, zig_zag_decode(3));

        assert_eq!(i32::MIN, zig_zag_decode(u32::MAX));
        assert_eq!(i32::MAX, zig_zag_decode(u32::MAX - 1));
    }
}
