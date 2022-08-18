use super::{LogVersion, ParseError, ParseResult};
use biterator::arbitrary_int::{u1, u2, UInt};
use biterator::Biterator;
use num_enum::TryFromPrimitive;
use std::array;
use std::io::Read;

#[derive(Debug)]
pub enum Decoded {
    Zero,
    U32(u32),
    I32(i32),
    TaggedVar([i32; 8]),
    Tagged32([i32; 3]),
    Tagged16([i16; 4]),
}

macro_rules! impl_from {
    ($from:ty, $variant:ident) => {
        impl From<$from> for Decoded {
            fn from(from: $from) -> Self {
                Self::$variant(from)
            }
        }
    };
}

impl_from!(u32, U32);
impl_from!(i32, I32);
impl_from!([i32; 8], TaggedVar);
impl_from!([i32; 3], Tagged32);
impl_from!([i16; 4], Tagged16);

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

pub fn read_uvar<R: Read>(data: &mut Biterator<R>) -> ParseResult<u32> {
    data.byte_align();

    // TODO: handle getting no bytes

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

        eprintln!();
    }

    // let mut continue_bit = true;
    // while continue_bit && data.peek_bit().is_some() {
    //     continue_bit = data.next_bit() == Some(u1::new(1));
    //
    //     uvar <<= 7;
    //
    //     // Safe because the biterator is guaranteed to have whole bytes after the call to byte_align()
    //     uvar += u32::from(data.next_bits::<7>().unwrap());
    // }

    // for byte in data.bytes() {
    //     uvar <<= 7;
    //     uvar += (byte & 0x7F) as u32;
    //
    //     // High bit is empty if this is the last byte
    //     if byte.leading_zeros() >= 1 {
    //         break;
    //     }
    // }

    Ok(uvar)
}

pub fn read_ivar<R: Read>(data: &mut Biterator<R>) -> ParseResult<i32> {
    Ok(zig_zag_decode(read_uvar(data)?))
}

pub fn read_negative_14_bit<R: Read>(data: &mut Biterator<R>) -> i32 {
    data.byte_align();

    unimplemented!();
}

fn bits_to_u32(acc: u32, bit: u1) -> u32 {
    (acc << 1) + u32::from(bit)
}

pub fn read_u32_elias_delta<R: Read>(data: &mut Biterator<R>) -> u32 {
    let mut bits = data.bits();

    let leading_zeros = bits.by_ref().take_while(|bit| *bit != u1::new(1)).count();

    // Initial 1 got consumed by `take_while` above
    let power = bits.by_ref().take(leading_zeros).fold(1, bits_to_u32);
    let power = power - 1;

    let remainder = bits.take(power as usize).fold(0, bits_to_u32);

    2_u32.pow(power) + remainder
}

pub fn read_i32_elias_delta<R: Read>(data: &mut Biterator<R>) -> i32 {
    zig_zag_decode(read_u32_elias_delta(data))
}

pub fn read_tagged_16<R: Read>(version: LogVersion, data: &mut Biterator<R>) -> [i16; 4] {
    data.byte_align();

    match version {
        LogVersion::V1 => read_tagged_16_v1(data),
        LogVersion::V2 => read_tagged_16_v2(data),
    }
}

const fn i4_to_i8(i4: u8) -> i8 {
    let byte = if (i4 & 8) > 0 { i4 & 0xF0 } else { i4 };
    byte as i8
}

// TODO: rewrite using new Biterator features
fn read_tagged_16_v1<R: Read>(data: &mut Biterator<R>) -> [i16; 4] {
    let tags = data.next_byte().unwrap();
    let mut result = [0; 4];

    let mut i = 0;
    while i < result.len() {
        let tag = tags & (0b11 << (i * 2));
        let tag = tag >> (i * 2);

        let next_byte = data.next_byte().unwrap();

        match tag {
            0 => {
                result[i] = 0;
            }
            1 => {
                result[i] = i4_to_i8(next_byte & 0x0F).into();
                i += 1;
                result[i] = i4_to_i8((next_byte & 0xF0) >> 4).into();
            }
            2 => {
                result[i] = (next_byte as i8).into();
            }
            3 => {
                result[i] =
                    i16::from_le_bytes([data.next_byte().unwrap(), data.next_byte().unwrap()]);
            }
            _ => unreachable!(),
        }

        i += 1;
    }

    result
}

fn read_tagged_16_v2<R: Read>(data: &mut Biterator<R>) -> [i16; 4] {
    assert!(data.peek_byte().is_some());

    // .unwrap guaranteed safe due to peek above
    let tags: [_; 4] = array::from_fn(|_| data.next_bits::<2>().unwrap());

    let mut result = [0; 4];
    for (i, result) in result.iter_mut().enumerate() {
        *result = match tags[i].value() {
            0 => 0,
            1 => i4_to_i8(data.next_nibble().unwrap().value()).into(),
            2 => data.next_byte().unwrap().into(),
            3 => {
                // TODO: use Biterator native 16 bit reads
                let byte1 = data.next_byte().unwrap();
                let byte2 = data.next_byte().unwrap();

                (i16::from(byte1) << 8) + i16::from(byte2)
            }
            4.. => unreachable!(),
        }
    }

    result
}

const fn zig_zag_decode(value: u32) -> i32 {
    (value >> 1) as i32 ^ -(value as i32 & 1)
}

#[cfg(test)]
mod test {
    use crate::{ParseError, ParseResult};
    use biterator::Biterator;

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
        fn read(bytes: [u8; 2]) -> u32 {
            super::read_u32_elias_delta(&mut Biterator::new(&bytes[..]))
        }

        assert_eq!(1, read([0x80, 0]));
        assert_eq!(2, read([0x40, 0]));
        assert_eq!(3, read([0x50, 0]));
        assert_eq!(16, read([0x28, 0]));
        assert_eq!(19, read([0x29, 0x80]));
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
