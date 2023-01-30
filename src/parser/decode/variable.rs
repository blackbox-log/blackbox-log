use super::zig_zag_decode;
use crate::parser::{InternalError, InternalResult};
use crate::Reader;

pub(crate) fn variable(data: &mut Reader) -> InternalResult<u32> {
    let mut uvar: u32 = 0;
    let mut offset: u32 = 0;
    loop {
        let byte = data.read_u8().ok_or(InternalError::Eof)?;
        let is_last_byte = (byte & 0x80) == 0;

        let byte = u32::from(byte & !0x80);
        uvar |= byte << offset;
        offset += 7;

        if !is_last_byte && offset >= 32 {
            return Err(InternalError::Retry);
        }

        if is_last_byte {
            break;
        }
    }

    Ok(uvar)
}

pub(crate) fn variable_signed(data: &mut Reader) -> InternalResult<i32> {
    variable(data).map(zig_zag_decode)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_ok(bytes: &[u8]) -> u32 {
        variable(&mut Reader::new(bytes)).unwrap()
    }

    #[test]
    fn zero() {
        assert_eq!(0, read_ok(&[0x00]));
        assert_eq!(0, read_ok(&[0x80, 0x00]));
    }

    #[test]
    fn one() {
        assert_eq!(1, read_ok(&[1]));
    }

    #[test]
    fn full_byte_output() {
        assert_eq!(0xFF, read_ok(&[0xFF, 0x01]));
    }

    #[test]
    fn max_two_byte_input() {
        assert_eq!(0x3FFF, read_ok(&[0xFF, 0x7F]));
    }

    #[test]
    fn max() {
        assert_eq!(0xFFFF_FFFF, read_ok(&[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]));
    }

    #[test]
    #[should_panic(expected = "Retry")]
    fn too_many_bytes() {
        assert_eq!(0xFFFF_FFFF, read_ok(&[0x80, 0x80, 0x80, 0x80, 0x80]));
    }
}
