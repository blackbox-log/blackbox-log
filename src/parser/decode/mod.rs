#![allow(clippy::cast_possible_truncation)]

mod negative_14_bit;
mod tagged_16;
mod tagged_32;
mod tagged_variable;
mod variable;

use alloc::vec::Vec;

pub(crate) use self::negative_14_bit::negative_14_bit;
pub(crate) use self::tagged_16::tagged_16;
pub(crate) use self::tagged_32::tagged_32;
pub(crate) use self::tagged_variable::tagged_variable;
pub(crate) use self::variable::{variable, variable_signed};
use super::InternalResult;
use crate::utils::{as_i32, as_u32};
use crate::Reader;

byte_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    pub(crate) enum Encoding {
        /// Signed variable byte
        VariableSigned = 0,
        /// Unsigned variable byte
        Variable = 1,
        /// Unsigned variable byte, but negated after decoding. Value fits in 14
        /// bits
        Negative14Bit = 3,
        TaggedVariable = 6,
        Tagged32 = 7,
        /// 1 tag byte containing 4 2 bit tags, followed by 4 fields
        ///
        /// | Tag | Field width         |
        /// |-----|---------------------|
        /// | 0   | 0 (field value = 0) |
        /// | 1   | 4                   |
        /// | 2   | 8                   |
        /// | 3   | 16                  |
        Tagged16 = 8,
        /// Nothing is written to the log, assume value is 0
        Null = 9,
    }
}

impl Encoding {
    pub(crate) const fn is_signed(&self) -> bool {
        match self {
            Self::VariableSigned
            | Self::Negative14Bit
            | Self::TaggedVariable
            | Self::Tagged32
            | Self::Tagged16 => true,
            Self::Variable | Self::Null => false,
        }
    }

    pub(crate) const fn max_chunk_size(&self) -> usize {
        match self {
            Self::TaggedVariable => 8,
            Self::Tagged32 => 3,
            Self::Tagged16 => 4,
            Self::VariableSigned | Self::Variable | Self::Negative14Bit | Self::Null => 1,
        }
    }

    pub(crate) fn decode_into(
        &self,
        data: &mut Reader,
        extra: usize,
        into: &mut Vec<u32>,
    ) -> InternalResult<()> {
        let range = 0..=extra;
        match self {
            Self::VariableSigned => into.push(as_u32(variable_signed(data)?)),
            Self::Variable => into.push(variable(data)?),

            Self::Negative14Bit => into.push(as_u32(negative_14_bit(data)?)),

            Self::TaggedVariable => {
                into.extend_from_slice(&tagged_variable(data, extra)?.map(as_u32)[range]);
            }
            Self::Tagged32 => into.extend_from_slice(&tagged_32(data)?.map(as_u32)[range]),
            Self::Tagged16 => {
                into.extend_from_slice(&tagged_16(data)?.map(|x| as_u32(x.into()))[range]);
            }

            Self::Null => into.push(0),
        };

        Ok(())
    }
}

#[inline]
const fn sign_extend<const BITS: u32>(from: u32) -> i32 {
    let unused_bits = 32 - BITS;
    as_i32(from << unused_bits) >> unused_bits
}

#[inline]
const fn zig_zag_decode(value: u32) -> i32 {
    as_i32(value >> 1) ^ -(as_i32(value) & 1)
}

#[cfg(test)]
mod tests {
    #[test]
    fn sign_extend() {
        use super::sign_extend;

        assert_eq!(0, sign_extend::<2>(0b00));
        assert_eq!(1, sign_extend::<2>(0b01));
        assert_eq!(-2, sign_extend::<2>(0b10));
        assert_eq!(-1, sign_extend::<2>(0b11));
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
