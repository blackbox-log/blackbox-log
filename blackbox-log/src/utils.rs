#[allow(clippy::cast_possible_wrap)]
#[inline(always)]
pub(crate) const fn as_signed(x: u32) -> i32 {
    x as i32
}

#[allow(clippy::cast_sign_loss)]
#[inline(always)]
pub(crate) const fn as_unsigned(x: i32) -> u32 {
    x as u32
}
