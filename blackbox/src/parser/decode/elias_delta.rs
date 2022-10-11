use bitter::BitReader;

use super::zig_zag_decode;
use crate::parser::{ParseError, ParseResult, Reader};

/// NB: May leave the bit stream unaligned
pub fn elias_delta(data: &mut Reader) -> ParseResult<u32> {
    let bits = data.bits();

    let leading_zeros = {
        let mut leading_zeros: u8 = 0;
        for _ in 0..6 {
            match bits.read_bit() {
                Some(false) => leading_zeros += 1,
                Some(_) => break,
                None => return Err(ParseError::UnexpectedEof),
            }
        }

        if leading_zeros > 5 {
            return Err(ParseError::Corrupted);
        }

        leading_zeros
    };

    let mut read = |count: u8| -> ParseResult<u32> {
        let count = count.into();

        if count == 0 {
            return Ok(0);
        }

        debug_assert!(count <= bitter::MAX_READ_BITS);

        let result = bits.read_bits(count).ok_or(ParseError::UnexpectedEof)?;
        let result = (1 << count) | result;
        Ok(result as u32 - 1)
    };

    let len = read(leading_zeros)? as u8;
    if len > 31 {
        return Err(ParseError::Corrupted);
    }

    let result = read(len)?;

    if result == (u32::MAX - 1) {
        // Use an extra bit to disambiguate (u32::MAX - 1) and u32::MAX
        let bit = bits.read_bit().ok_or(ParseError::UnexpectedEof)?;
        Ok(result + u32::from(bit))
    } else {
        Ok(result)
    }
}

/// NB: May leave the bit stream unaligned
pub fn elias_delta_signed(data: &mut Reader) -> ParseResult<i32> {
    elias_delta(data).map(zig_zag_decode)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unaligned_min() {
        let mut bits = Reader::new(&[1]);
        bits.bits().read_bits(7);
        assert_eq!(0, super::elias_delta(&mut bits).unwrap());
    }

    #[test]
    fn unsigned() {
        fn read(bytes: &[u8]) -> u32 {
            super::elias_delta(&mut Reader::new(bytes)).unwrap()
        }

        assert_eq!(0, read(&[0x80, 0]));
        assert_eq!(1, read(&[0x40, 0]));
        assert_eq!(2, read(&[0x50, 0]));
        assert_eq!(15, read(&[0x28, 0]));
        assert_eq!(18, read(&[0x29, 0x80]));

        assert_eq!(8191, read(&[0x1C, 0, 0]));
    }

    #[test]
    fn unsigned_max() {
        let bytes = &[0x04, 0x1F, 0xFF, 0xFF, 0xFF, 0xE0];
        let mut bits = Reader::new(bytes.as_slice());

        assert_eq!(u32::MAX, super::elias_delta(&mut bits).unwrap());
    }

    #[test]
    fn signed() {
        fn read(bytes: &[u8]) -> i32 {
            super::elias_delta_signed(&mut Reader::new(bytes)).unwrap()
        }

        assert_eq!(0, read(&[0x80, 0]));
        assert_eq!(-1, read(&[0x40, 0]));
        assert_eq!(1, read(&[0x50, 0]));
        assert_eq!(-8, read(&[0x28, 0]));
    }

    #[test]
    fn signed_min() {
        let bytes = &[0x04, 0x1F, 0xFF, 0xFF, 0xFF, 0xE0];
        let mut bits = Reader::new(bytes.as_slice());

        assert_eq!(i32::MIN, super::elias_delta_signed(&mut bits).unwrap());
    }

    #[test]
    fn signed_max() {
        let bytes = &[0x04, 0x1F, 0xFF, 0xFF, 0xFF, 0xC0];
        let mut bits = Reader::new(bytes.as_slice());

        assert_eq!(i32::MAX, super::elias_delta_signed(&mut bits).unwrap());
    }

    #[test]
    #[should_panic(expected = "UnexpectedEof")]
    fn no_data() {
        let mut bits = Reader::new(&[]);
        super::elias_delta(&mut bits).unwrap();
    }

    #[test]
    fn too_many_leading_zeros() {
        let mut bits = Reader::new(&[0b0000_0010]);
        let result = super::elias_delta(&mut bits);
        assert!(matches!(result, Err(ParseError::Corrupted)));
    }

    #[test]
    #[should_panic(expected = "UnexpectedEof")]
    fn too_few_middle_bits() {
        let mut bits = Reader::new(&[6]);
        super::elias_delta(&mut bits).unwrap();
    }

    #[test]
    fn too_many_middle_bits() {
        let mut bits = Reader::new(&[6, 0]);
        let result = super::elias_delta(&mut bits);
        assert!(matches!(result, Err(ParseError::Corrupted)));
    }

    #[test]
    #[should_panic(expected = "UnexpectedEof")]
    fn too_few_remainder_bits() {
        let mut bits = Reader::new(&[0x36]);
        super::elias_delta(&mut bits).unwrap();
    }

    #[test]
    #[should_panic(expected = "UnexpectedEof")]
    fn missing_disambiguation_bit() {
        let mut bits = Reader::new(&[0, 0x10, 0x7F, 0xFF, 0xFF, 0xFF]);
        bits.bits().read_bits(6);
        super::elias_delta(&mut bits).unwrap();
    }
}
