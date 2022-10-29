use crate::parser::{ParseError, ParseResult, Reader};

pub fn tagged_variable(data: &mut Reader, extra: usize) -> ParseResult<[i32; 8]> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn one_value() {
        let mut b = Reader::new(&[2]);

        let expected = [1, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(expected, tagged_variable(&mut b, 0).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn simple_two_values() {
        let b = [0b0000_0011, 2, 2];
        let mut b = Reader::new(&b);

        let expected = [1, 1, 0, 0, 0, 0, 0, 0];
        assert_eq!(expected, tagged_variable(&mut b, 1).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    fn fewer_in_tag_than_expected() {
        let b = [0b0000_0010, 2];
        let mut b = Reader::new(&b);

        let expected = [0, 1, 0, 0, 0, 0, 0, 0];
        assert_eq!(expected, tagged_variable(&mut b, 1).unwrap());
        assert!(b.is_empty());
    }

    #[test]
    #[should_panic(expected = "UnexpectedEof")]
    fn multiple_expected_but_empty() {
        let mut b = Reader::new(&[]);

        tagged_variable(&mut b, 1).unwrap();
    }

    #[test]
    #[should_panic(expected = "Corrupted")]
    fn more_in_tag_than_expected() {
        let b = [0b0000_0111, 2, 2, 2];
        let mut b = Reader::new(&b);

        tagged_variable(&mut b, 1).unwrap();
    }
}
