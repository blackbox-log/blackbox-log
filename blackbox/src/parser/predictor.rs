use core::ops::{Add, Shr};

use num_enum::TryFromPrimitive;
use num_traits::ops::checked::CheckedSub;

use super::{as_signed, as_unsigned, Headers, ParseResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum Predictor {
    Zero = 0,
    Previous,
    StraightLine,
    Average2,
    MinThrottle,
    Motor0,
    Increment,
    HomeLat, // TODO: check that lat = 0, lon = 1
    FifteenHundred,
    VBatReference,
    LastMainFrameTime,
    MinMotor,
    // HomeLon = 256,
}

impl Predictor {
    #[allow(clippy::too_many_arguments)]
    pub fn apply(
        self,
        headers: &Headers,
        value: u32,
        signed: bool,
        current: &[u32],
        last: Option<u32>,
        last_last: Option<u32>,
        skipped_frames: u32,
    ) -> ParseResult<u32> {
        let _span = if signed {
            let value = as_signed(value);
            tracing::trace_span!(
                "Predictor::apply",
                ?self,
                value,
                last = last.map(as_signed),
                last_last = last_last.map(as_signed),
                skipped_frames,
            )
        } else {
            tracing::trace_span!(
                "Predictor::apply",
                ?self,
                value,
                last,
                last_last,
                skipped_frames
            )
        };
        let _span = _span.enter();

        let diff = match self {
            Self::Zero => 0,
            Self::Previous => last.unwrap_or(0),
            Self::StraightLine => {
                if signed {
                    as_unsigned(straight_line::<i32>(
                        last.map(as_signed),
                        last_last.map(as_signed),
                    ))
                } else {
                    straight_line::<u32>(last, last_last)
                }
            }
            Self::Average2 => {
                if signed {
                    as_unsigned(average_2(last.map(as_signed), last_last.map(as_signed)))
                } else {
                    average_2(last, last_last)
                }
            }
            Self::MinThrottle => headers.min_throttle.into(),
            Self::Motor0 => headers.main_frames.get_motor_0_from(current)?,
            Self::Increment => {
                if signed {
                    1 + skipped_frames + last.unwrap_or(0)
                } else {
                    let skipped_frames = i32::try_from(skipped_frames)
                        .expect("never skip more than i32::MAX frames");
                    as_unsigned(1 + skipped_frames + as_signed(last.unwrap_or(0)))
                }
            }
            // Self::HomeLat => todo!(), // TODO: check that lat = 0, lon = 1
            Self::FifteenHundred => 1500,
            Self::VBatReference => headers.vbat_reference.into(),
            // Self::LastMainFrameTime => todo!(),
            Self::MinMotor => headers.motor_output_range.min().into(),
            // Self::HomeLon => todo!(),
            Self::HomeLat | Self::LastMainFrameTime => {
                tracing::warn!("found unimplemented predictor: {self:?}");
                0
            }
        };

        Ok(if signed {
            let signed = as_signed(value) + as_signed(diff);
            tracing::trace!(return = signed);
            as_unsigned(signed)
        } else {
            let x = value + diff;
            tracing::trace!(return = x);
            x
        })
    }
}

#[inline]
pub(crate) fn straight_line<T>(last: Option<T>, last_last: Option<T>) -> T
where
    T: Copy + Default + Add<Output = T> + CheckedSub<Output = T>,
{
    match (last, last_last) {
        (Some(last), Some(last_last)) => last.checked_sub(&last_last).unwrap_or_default() + last,
        (Some(last), None) => last,
        _ => T::default(),
    }
}

#[inline]
pub(crate) fn average_2<T>(last: Option<T>, last_last: Option<T>) -> T
where
    T: Copy + Default + Add<Output = T> + Shr<usize, Output = T> + From<u8>,
{
    let last = last.unwrap_or_default();
    last_last.map_or(last, |last_last| (last + last_last) >> 1)
}

#[cfg(test)]
mod tests {
    use test_case::case;

    #[case(None, None => 0)]
    #[case(Some(10), None => 10)]
    #[case(Some(-2), None => -2)]
    #[case(Some(12), Some(10) => 14)]
    #[case(Some(10), Some(12) => 8)]
    #[case(Some(0), Some(i8::MIN) => 0 ; "underflow")]
    fn straight_line(last: Option<i8>, last_last: Option<i8>) -> i8 {
        super::straight_line(last, last_last)
    }
}
