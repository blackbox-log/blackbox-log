use core::ops::{Add, Sub};

use num_enum::TryFromPrimitive;

use super::{Headers, ParseResult};

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
    pub fn apply(
        self,
        headers: &Headers,
        value: i64,
        current: &[i64],
        last: Option<i64>,
        last_last: Option<i64>,
        skipped_frames: i64,
    ) -> ParseResult<i64> {
        let diff = match self {
            Self::Zero => 0,
            Self::Previous => last.unwrap_or(0),
            Self::StraightLine => straight_line(last, last_last),
            Self::Average2 => (last.unwrap_or(0) + last_last.unwrap_or(0)) / 2,
            Self::MinThrottle => headers.min_throttle.into(),
            Self::Motor0 => headers.main_frames.get_motor_0_from(current)?,
            Self::Increment => 1 + skipped_frames + last.unwrap_or(0),
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

        Ok(value + diff)
    }
}

#[inline]
pub(crate) fn straight_line<T>(last: Option<T>, last_last: Option<T>) -> T
where
    T: Copy + Add<Output = T> + Sub<Output = T> + From<bool>,
{
    match (last, last_last) {
        (Some(last), Some(last_last)) => (last - last_last) + last,
        (Some(last), None) => last,
        _ => false.into(),
    }
}

#[cfg(test)]
mod tests {
    use test_case::case;

    #[case(None, None => 0)]
    #[case(Some(10), None => 10)]
    #[case(Some(-2), None => -2)]
    #[case(Some(12), Some(10) => 14)]
    #[case(Some(10), Some(12) => 8)]
    fn straight_line(last: Option<i8>, last_last: Option<i8>) -> i8 {
        super::straight_line(last, last_last)
    }
}
