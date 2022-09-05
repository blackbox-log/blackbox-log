use super::zig_zag_decode;
use crate::{ParseError, ParseResult, Reader};
use bitter::BitReader;
use tracing::instrument;

#[instrument(level = "trace", skip(data), ret)]
/// NB: May leave the bit stream unaligned
pub fn read_u32_elias_delta(data: &mut Reader) -> ParseResult<u32> {
    let leading_zeros = {
        let mut leading_zeros: u8 = 0;
        for _ in 0..6 {
            match data.read_bit() {
                Some(false) => leading_zeros += 1,
                Some(_) => break,
                None => return Err(ParseError::unexpected_eof()),
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

        if data.refill_lookahead() < count {
            return Err(ParseError::unexpected_eof());
        }

        let result = (1 << count) | data.peek(count);
        data.consume(count);
        Ok(result as u32 - 1)
    };

    // Guaranteed to be <= 31 since we're reading at most 5 bits
    let len = read(leading_zeros)? as u8;
    if len > 31 {
        return Err(ParseError::Corrupted);
    }

    let result = read(len)?;

    if result == (u32::MAX - 1) {
        // Use an extra bit to disambiguate (u32::MAX - 1) and u32::MAX
        let bit: u32 = data.read_bit().ok_or(ParseError::Corrupted)?.into();
        Ok(result + bit)
    } else {
        Ok(result)
    }
}

#[instrument(level = "trace", skip(data), ret)]
/// NB: May leave the bit stream unaligned
pub fn read_i32_elias_delta(data: &mut Reader) -> ParseResult<i32> {
    read_u32_elias_delta(data).map(zig_zag_decode)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unsigned() {
        fn read(bytes: &[u8]) -> u32 {
            super::read_u32_elias_delta(&mut Reader::new(bytes)).unwrap()
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

        assert_eq!(u32::MAX, super::read_u32_elias_delta(&mut bits).unwrap());
    }

    #[test]
    fn signed() {
        fn read(bytes: &[u8]) -> i32 {
            super::read_i32_elias_delta(&mut Reader::new(bytes)).unwrap()
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

        assert_eq!(i32::MIN, super::read_i32_elias_delta(&mut bits).unwrap());
    }

    #[test]
    fn signed_max() {
        let bytes = &[0x04, 0x1F, 0xFF, 0xFF, 0xFF, 0xC0];
        let mut bits = Reader::new(bytes.as_slice());

        assert_eq!(i32::MAX, super::read_i32_elias_delta(&mut bits).unwrap());
    }
}
