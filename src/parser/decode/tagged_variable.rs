use crate::parser::{InternalError, InternalResult};
use crate::utils::as_u32;
use crate::Reader;

pub(crate) fn tagged_variable(data: &mut Reader, out: &mut [u32]) -> InternalResult<()> {
    debug_assert!(!out.is_empty() && out.len() <= 8);

    if out.len() == 1 {
        out[0] = as_u32(super::variable_signed(data)?);
    } else {
        let mut header = data.read_u8().ok_or(InternalError::Eof)?;

        for value in out {
            *value = if (header & 1) == 1 {
                as_u32(super::variable_signed(data)?)
            } else {
                0
            };

            header >>= 1;
        }

        if header != 0 {
            return Err(InternalError::Retry);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_value() {
        let mut b = Reader::new(&[2]);

        let mut out = [0];
        tagged_variable(&mut b, &mut out).unwrap();
        assert_eq!([1], out);
        assert!(b.is_empty());
    }

    #[test]
    fn simple_two_values() {
        let b = [0b0000_0011, 2, 2];
        let mut b = Reader::new(&b);

        let mut out = [0; 2];
        tagged_variable(&mut b, &mut out).unwrap();
        assert_eq!([1, 1], out);
        assert!(b.is_empty());
    }

    #[test]
    fn fewer_in_tag_than_expected() {
        let b = [0b0000_0010, 2];
        let mut b = Reader::new(&b);

        let mut out = [0; 2];
        tagged_variable(&mut b, &mut out).unwrap();
        assert_eq!([0, 1], out);
        assert!(b.is_empty());
    }

    #[test]
    #[should_panic(expected = "Eof")]
    fn multiple_expected_but_empty() {
        let mut b = Reader::new(&[]);

        tagged_variable(&mut b, &mut [0; 2]).unwrap();
    }

    #[test]
    #[should_panic(expected = "Retry")]
    fn more_in_tag_than_expected() {
        let b = [0b0000_0111, 2, 2, 2];
        let mut b = Reader::new(&b);

        tagged_variable(&mut b, &mut [0; 2]).unwrap();
    }
}
