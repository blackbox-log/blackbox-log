#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]

mod elias_delta;
mod negative_14_bit;
mod tagged_16;
mod tagged_32;
mod tagged_variable;
mod variable;

use alloc::vec;
use alloc::vec::Vec;
pub use elias_delta::{elias_delta, elias_delta_signed};
pub use negative_14_bit::negative_14_bit;
pub use tagged_16::tagged_16;
pub use tagged_32::tagged_32;
pub use tagged_variable::tagged_variable;
pub use variable::{variable, variable_signed};

use super::{ParseResult, Reader};
use crate::LogVersion;
use num_enum::TryFromPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum Encoding {
    /// Signed variable byte
    VariableSigned = 0,
    /// Unsigned variable byte
    Variable = 1,
    /// Unsigned variable byte, but negated after decoding. Value fits in 14 bits
    Negative14Bit = 3,
    EliasDelta = 4,
    EliasDeltaSigned = 5,
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
    EliasGamma = 10,
    EliasGammaSigned = 11,
}

impl Encoding {
    pub(crate) fn max_chunk_size(&self) -> usize {
        match self {
            Self::TaggedVariable => 8,
            Self::Tagged32 => 3,
            Self::Tagged16 => 4,
            Self::VariableSigned
            | Self::Variable
            | Self::Negative14Bit
            | Self::EliasDelta
            | Self::EliasDeltaSigned
            | Self::Null
            | Self::EliasGamma
            | Self::EliasGammaSigned => 1,
        }
    }

    pub(crate) fn decode(
        &self,
        data: &mut Reader,
        version: LogVersion,
        extra: usize,
    ) -> ParseResult<Vec<i64>> {
        let range = 0..=extra;
        let values = match self {
            Self::VariableSigned => vec![variable_signed(data)?.into()],
            Self::Variable => vec![variable(data)?.into()],

            Self::Negative14Bit => vec![negative_14_bit(data)?.into()],

            Self::EliasDelta => vec![elias_delta(data)?.into()],
            Self::EliasDeltaSigned => vec![elias_delta_signed(data)?.into()],

            Self::TaggedVariable => tagged_variable(data, extra)?.map(Into::into)[range].to_vec(),
            Self::Tagged32 => tagged_32(data)?.map(Into::into)[range].to_vec(),
            Self::Tagged16 => tagged_16(version, data)?.map(Into::into)[range].to_vec(),

            Self::Null => vec![0],

            Self::EliasGamma | Self::EliasGammaSigned => todo!(),
        };

        Ok(values)
    }
}

#[inline]
const fn sign_extend<const BITS: u32>(from: u32) -> i32 {
    let unused_bits = 32 - BITS;
    (from << unused_bits) as i32 >> unused_bits
}

#[inline]
const fn zig_zag_decode(value: u32) -> i32 {
    (value >> 1) as i32 ^ -(value as i32 & 1)
}

#[cfg(test)]
mod test {
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
