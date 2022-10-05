#![allow(dead_code)]

#[inline]
pub fn sign_extend_24_bit(x: u32) -> i32 {
    unsafe { ffi::signExtend24Bit(x) }
}

#[inline]
pub fn sign_extend_14_bit(x: u16) -> i32 {
    unsafe { ffi::signExtend14Bit(x) }
}

#[inline]
pub fn sign_extend_6_bit(x: u8) -> i32 {
    unsafe { ffi::signExtend6Bit(x) }
}

#[inline]
pub fn sign_extend_4_bit(x: u8) -> i32 {
    unsafe { ffi::signExtend4Bit(x) }
}

#[inline]
pub fn sign_extend_2_bit(x: u8) -> i32 {
    unsafe { ffi::signExtend2Bit(x) }
}

#[inline]
pub fn zigzag_encode(value: i32) -> u32 {
    unsafe { ffi::zigzagEncode(value) }
}

#[inline]
pub fn zigzag_decode(value: u32) -> i32 {
    unsafe { ffi::zigzagDecode(value) }
}

mod ffi {
    #[allow(non_snake_case)]
    extern "C" {
        // pub(super)fn intToFloat(i: i32) -> f32;
        // pub(super)fn uintToFloat(u: u32) -> f32;
        // pub(super)fn floatToInt(f: f32) -> i32;
        // pub(super)fn floatToUint(f: f32) -> u32;
        pub(super) fn signExtend24Bit(u: u32) -> i32;
        pub(super) fn signExtend14Bit(word: u16) -> i32;
        pub(super) fn signExtend6Bit(byte: u8) -> i32;
        pub(super) fn signExtend4Bit(nibble: u8) -> i32;
        pub(super) fn signExtend2Bit(byte: u8) -> i32;
        pub(super) fn zigzagEncode(value: i32) -> u32;
        pub(super) fn zigzagDecode(value: u32) -> i32;
    }
}
