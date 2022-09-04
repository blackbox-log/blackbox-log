use super::zig_zag_decode;
use crate::{ParseError, ParseResult, Reader};
use bitter::BitReader;
use tracing::instrument;

#[allow(clippy::assertions_on_constants)]
#[instrument(level = "trace", skip(data), ret)]
pub fn read_uvar(data: &mut Reader) -> ParseResult<u32> {
    // FIXME: data.byte_align();

    // 32 bits at 7 bits / byte = 5 bytes
    const _: () = assert!((5 * 8) <= bitter::MAX_READ_BITS, "bit buffer is too small");

    data.refill_lookahead();

    let mut uvar: u32 = 0;
    let mut offset: u32 = 0;
    loop {
        if !data.has_bits_remaining(8) {
            return Err(ParseError::unexpected_eof());
        }

        let is_last_byte = data.peek(1) == 0;
        data.consume(1);

        uvar |= (data.peek(7) << offset) as u32;
        data.consume(7);

        offset += 7;

        if is_last_byte || offset >= 32 {
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
}
