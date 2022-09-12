use bitter::BitReader;

use crate::parser::{ParseError, ParseResult};
use crate::Reader;

pub fn tagged_variable(data: &mut Reader, extra: usize) -> ParseResult<[i32; 8]> {
    debug_assert!(data.byte_aligned());
    debug_assert!(extra < 8);

    let mut values = [0; 8];

    if extra == 0 {
        values[0] = super::variable_signed(data)?;
    } else {
        let mut header = data.read_u8().ok_or(ParseError::UnexpectedEof)?;

        for value in values.iter_mut().take(extra + 1) {
            *value = if (header & 1) == 1 {
                super::variable_signed(data)?
            } else {
                0
            };

            header >>= 1;
        }

        if header != 0 {
            return Err(ParseError::Corrupted);
        }
    }

    Ok(values)
}
