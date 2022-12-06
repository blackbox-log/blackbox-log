use core::ops::{Add, Div, Sub};

use super::frame::GpsPosition;
use crate::utils::{as_i32, as_u32};
use crate::Headers;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub(crate) enum Predictor {
    Zero = 0,
    Previous,
    StraightLine,
    Average2,
    MinThrottle,
    Motor0,
    Increment,
    HomeLat,
    FifteenHundred,
    VBatReference,
    LastMainFrameTime,
    MinMotor,
    HomeLon = 256,
}

impl Predictor {
    pub(crate) fn apply(
        self,
        value: u32,
        signed: bool,
        current: Option<&[u32]>,
        ctx: &PredictorContext,
    ) -> u32 {
        let _span = if signed {
            tracing::trace_span!(
                "Predictor::apply",
                ?self,
                value = as_i32(value),
                last = ctx.last.map(as_i32),
                last_last = ctx.last_last.map(as_i32),
                skipped_frames = ctx.skipped_frames,
            )
        } else {
            tracing::trace_span!(
                "Predictor::apply",
                ?self,
                value,
                last = ctx.last,
                last_last = ctx.last_last,
                skipped_frames = ctx.skipped_frames
            )
        };
        let _span = _span.enter();

        let diff = match self {
            Self::Zero => 0,
            Self::Previous => ctx.last.unwrap_or(0),
            Self::StraightLine => {
                if signed {
                    as_u32(straight_line(
                        ctx.last.map(as_i32),
                        ctx.last_last.map(as_i32),
                    ))
                } else {
                    straight_line(ctx.last, ctx.last_last)
                }
            }
            Self::Average2 => {
                if signed {
                    as_u32(average(ctx.last.map(as_i32), ctx.last_last.map(as_i32)))
                } else {
                    average(ctx.last, ctx.last_last)
                }
            }
            Self::MinThrottle => ctx.headers.min_throttle.unwrap().into(),
            Self::Motor0 => current.map_or_else(
                || {
                    tracing::debug!("found {self:?} without current values");
                    0
                },
                |current| ctx.headers.main_frames.get_motor_0_from(current),
            ),
            Self::Increment => {
                if signed {
                    ctx.skipped_frames
                        .wrapping_add(1)
                        .wrapping_add(ctx.last.unwrap_or(0))
                } else {
                    let skipped_frames = i32::try_from(ctx.skipped_frames)
                        .expect("never skip more than i32::MAX frames");
                    as_u32(
                        skipped_frames
                            .wrapping_add(1)
                            .wrapping_add(as_i32(ctx.last.unwrap_or(0))),
                    )
                }
            }
            Self::HomeLat | Self::HomeLon => ctx.gps_home.map_or_else(
                || {
                    tracing::debug!("found {self:?} without gps home");
                    // TODO: invalidate result
                    0
                },
                |home| {
                    as_u32(if self == Self::HomeLat {
                        home.latitude
                    } else {
                        home.longitude
                    })
                },
            ),
            Self::FifteenHundred => 1500,
            Self::VBatReference => ctx.headers.vbat_reference.unwrap().into(),
            Self::LastMainFrameTime => {
                tracing::debug!("found unhandled {self:?}");
                0
            }
            Self::MinMotor => ctx.headers.motor_output_range.unwrap().min.into(),
        };

        if signed {
            let signed = as_i32(value).wrapping_add(as_i32(diff));
            tracing::trace!(return = signed);
            as_u32(signed)
        } else {
            let x = value.wrapping_add(diff);
            tracing::trace!(return = x);
            x
        }
    }

    pub(crate) fn from_num_str(s: &str) -> Option<Self> {
        match s {
            "0" => Some(Self::Zero),
            "1" => Some(Self::Previous),
            "2" => Some(Self::StraightLine),
            "3" => Some(Self::Average2),
            "4" => Some(Self::MinThrottle),
            "5" => Some(Self::Motor0),
            "6" => Some(Self::Increment),
            "7" => Some(Self::HomeLat), // TODO: check that lat = 0, lon = 1
            "8" => Some(Self::FifteenHundred),
            "9" => Some(Self::VBatReference),
            "10" => Some(Self::LastMainFrameTime),
            "11" => Some(Self::MinMotor),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PredictorContext<'a, 'data> {
    headers: &'a Headers<'data>,
    last: Option<u32>,
    last_last: Option<u32>,
    skipped_frames: u32,
    gps_home: Option<GpsPosition>,
}

impl<'a, 'data> PredictorContext<'a, 'data> {
    pub(crate) const fn new(headers: &'a Headers<'data>) -> Self {
        Self {
            headers,
            last: None,
            last_last: None,
            skipped_frames: 0,
            gps_home: None,
        }
    }

    pub(crate) const fn with_skipped(headers: &'a Headers<'data>, skipped_frames: u32) -> Self {
        Self {
            headers,
            last: None,
            last_last: None,
            skipped_frames,
            gps_home: None,
        }
    }

    pub(crate) const fn with_home(
        headers: &'a Headers<'data>,
        gps_home: Option<GpsPosition>,
    ) -> Self {
        Self {
            headers,
            last: None,
            last_last: None,
            skipped_frames: 0,
            gps_home,
        }
    }

    pub(crate) fn set_last(&mut self, last: Option<u32>) -> &mut Self {
        self.last = last;
        self
    }

    pub(crate) fn set_last_2(&mut self, last: Option<u32>, last_last: Option<u32>) -> &mut Self {
        self.last = last;
        self.last_last = last_last;
        self
    }
}

#[inline]
pub(crate) fn straight_line<T>(last: Option<T>, last_last: Option<T>) -> T
where
    T: TemporaryOverflow + Default,
    T::Wide: Copy + Sub<Output = T::Wide> + Add<Output = T::Wide> + PartialOrd,
{
    match (last, last_last) {
        (Some(last), Some(last_last)) => {
            let fallback = last;

            let result = {
                let last = last.widen();
                let last_last = last_last.widen();
                let sum = last + last;

                // Work around not being able to use .checked_sub()
                if !T::SIGNED && last_last > sum {
                    return fallback;
                }

                sum - last_last
            };
            T::try_from(result).unwrap_or(fallback)
        }
        (Some(last), None) => last,
        _ => T::default(),
    }
}

#[inline]
fn average<T>(last: Option<T>, last_last: Option<T>) -> T
where
    T: TemporaryOverflow + Copy + Default,
    T::Wide: Add<Output = T::Wide> + Div<Output = T::Wide> + From<u8>,
{
    let last = last.unwrap_or_default();
    last_last.map_or(last, |last_last| {
        T::truncate_from((last.widen() + last_last.widen()) / 2.into())
    })
}

pub(crate) trait TemporaryOverflow
where
    Self: Copy + TryFrom<Self::Wide>,
    Self::Wide: From<Self>,
{
    const SIGNED: bool;
    type Wide;
    fn truncate_from(larger: Self::Wide) -> Self;

    #[inline]
    fn widen(self) -> Self::Wide {
        self.into()
    }
}

macro_rules! impl_next_larger {
    ($($sign:ident $base:ident -> $wide:ident),+ $(,)?) => {
        $(impl_next_larger!($sign, $base, $wide);)+
    };
    (signed, $base:ident, $wide:ident) => { impl_next_larger!(true, $base, $wide); };
    (unsigned, $base:ident, $wide:ident) => { impl_next_larger!(false, $base, $wide); };
    ($sign:expr, $base:ident, $wide:ident) => {
        impl TemporaryOverflow for $base {
            const SIGNED: bool = $sign;
            type Wide = $wide;

            #[inline]
            fn truncate_from(wide: Self::Wide) -> Self {
                wide as $base
            }
        }
    }
}

impl_next_larger!(unsigned u8 -> u16, signed i8 -> i16);
impl_next_larger!(unsigned u16 -> u32, signed i16 -> i32);
impl_next_larger!(unsigned u32 -> u64, signed i32 -> i64);
impl_next_larger!(unsigned u64 -> u128, signed i64 -> i128);

#[cfg(test)]
mod tests {
    use test_case::case;

    #[case(None, None => 0)]
    #[case(Some(10), None => 10)]
    #[case(Some(-2), None => -2)]
    #[case(Some(12), Some(10) => 14)]
    #[case(Some(10), Some(12) => 8)]
    #[case(Some(0), Some(i8::MAX) => -i8::MAX)]
    #[case(Some(0), Some(i8::MIN) => 0 ; "underflow")]
    #[case(Some(126), Some(0) => 126 ; "overflow")]
    fn straight_line_signed(last: Option<i8>, last_last: Option<i8>) -> i8 {
        super::straight_line(last, last_last)
    }

    #[case(Some(2),Some(2) => 2)]
    #[case(Some(12), Some(10) => 14)]
    #[case(Some(10), Some(12) => 8)]
    #[case(Some(0), Some(u8::MIN) => 0 ; "underflow")]
    #[case(Some(u8::MAX - 1), Some(0) => 254 ; "overflow")]
    #[case(Some(0), Some(u8::MAX) => 0 ; "negative result")]
    fn straight_line_unsigned(last: Option<u8>, last_last: Option<u8>) -> u8 {
        super::straight_line(last, last_last)
    }

    #[case(None, None => 0)]
    #[case(Some(-1), None => -1)]
    #[case(Some(2), Some(-1) => 0)]
    #[case(Some(i32::MAX), Some(1) => 0x4000_0000 ; "overflow")]
    fn average_signed(last: Option<i32>, last_last: Option<i32>) -> i32 {
        super::average(last, last_last)
    }

    #[case(None, None => 0)]
    #[case(Some(1), None => 1)]
    #[case(Some(2), Some(10) => 6)]
    #[case(Some(u32::MAX), Some(1) => 0x8000_0000 ; "overflow")]
    fn average_unsigned(last: Option<u32>, last_last: Option<u32>) -> u32 {
        super::average(last, last_last)
    }
}
