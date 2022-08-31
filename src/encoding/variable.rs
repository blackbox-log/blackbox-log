use super::zig_zag_decode;
use crate::{ParseError, ParseResult};
use biterator::Biterator;
use std::io::Read;
use tracing::instrument;

#[instrument(level = "trace", skip(data), ret)]
pub fn read_uvar<R: Read>(data: &mut Biterator<R>) -> ParseResult<u32> {
    data.byte_align();

    let mut uvar: u32 = 0;
    for i in 0.. {
        let is_last_byte = match data.next_bit() {
            Some(bit) => bit.get() == 0,
            None => return Err(ParseError::unexpected_eof()),
        };

        // Unwrap is safe after byte_align() above
        let byte = data.next_bits::<7>().unwrap();
        let byte = u32::from(byte.get())
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

#[cfg(test)]
mod test {
    use super::*;

    fn read_ok(bytes: &[u8]) -> u32 {
        super::read_uvar(&mut Biterator::new(bytes)).unwrap()
    }

    fn read_err(bytes: &[u8]) -> ParseError {
        super::read_uvar(&mut Biterator::new(bytes)).unwrap_err()
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
