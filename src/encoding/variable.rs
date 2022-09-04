use super::zig_zag_decode;
use crate::{ParseError, ParseResult, Reader};
use bitter::BitReader;
use tracing::instrument;

#[instrument(level = "trace", skip(data), ret)]
pub fn read_uvar(data: &mut Reader) -> ParseResult<u32> {
    // FIXME: data.byte_align();

    let mut uvar: u32 = 0;
    for i in 0.. {
        if !data.has_bits_remaining(8) {
            return Err(ParseError::unexpected_eof());
        }

        let is_last_byte = !data.read_bit().unwrap();

        let byte = data.read_bits(7).unwrap();
        let byte = (byte as u32)
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
pub fn read_ivar(data: &mut Reader) -> ParseResult<i32> {
    read_uvar(data).map(zig_zag_decode)
}

#[cfg(test)]
mod test {
    use super::*;

    fn read_ok(bytes: &[u8]) -> u32 {
        super::read_uvar(&mut Reader::new(bytes)).unwrap()
    }

    fn read_err(bytes: &[u8]) -> ParseError {
        super::read_uvar(&mut Reader::new(bytes)).unwrap_err()
    }

    #[test]
    fn read() {
        assert_eq!(0, read_ok(&[0x00]));
        assert_eq!(0, read_ok(&[0x80, 0x00]));
        assert_eq!(1, read_ok(&[1]));
        assert_eq!(0xFF, read_ok(&[0xFF, 0x01]));
        assert_eq!(0x3FFF, read_ok(&[0xFF, 0x7F]));
    }

    #[test]
    fn max() {
        assert_eq!(0xFFFF_FFFF, read_ok(&[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]));
        assert_eq!(0xFFFF_FFFF, read_ok(&[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]));
    }

    #[test]
    fn corrupted() {
        let err = read_err(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        assert!(matches!(err, ParseError::Corrupted));
    }
}
