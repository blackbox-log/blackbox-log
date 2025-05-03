use core::ops::{Add, Div, Sub};

use super::frame::GpsPosition;
use crate::data_v2::History;
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
        _value: u32,
        _signed: bool,
        _current: Option<&[u32]>,
        _ctx: &PredictorContext,
    ) -> u32 {
        todo!()
    }

    pub(crate) fn apply_v2(
        self,
        value: u32,
        signed: bool,
        skipped: u32,
        ctx: &PredictorContextV2,
        history: History<u32>,
    ) -> u32 {
        let diff = match self {
            Self::Zero => 0,
            Self::Previous => history.last_or(0),
            Self::StraightLine => {
                if signed {
                    as_u32(straight_line(history.as_i32()))
                } else {
                    straight_line(history)
                }
            }
            Self::Average2 => {
                if signed {
                    as_u32(average(history.as_i32()))
                } else {
                    average(history)
                }
            }
            Self::MinThrottle => ctx.min_throttle,
            Self::Motor0 => ctx.motor_0,
            Self::Increment => {
                // FIXME: switched?
                if signed {
                    skipped.wrapping_add(1).wrapping_add(history.last_or(0))
                } else {
                    let skipped =
                        i32::try_from(skipped).expect("never skip more than i32::MAX frames");
                    as_u32(
                        skipped
                            .wrapping_add(1)
                            .wrapping_add(as_i32(history.last_or(0))),
                    )
                }
            }
            Self::HomeLat | Self::HomeLon => as_u32(if self == Self::HomeLat {
                ctx.gps_home.latitude
            } else {
                ctx.gps_home.longitude
            }),
            Self::FifteenHundred => 1500,
            Self::VBatReference => ctx.vbat_reference,
            Self::LastMainFrameTime => {
                // TODO
                tracing::debug!("found unhandled {self:?}");
                0
            }
            Self::MinMotor => ctx.min_motor,
        };

        if signed {
            as_u32(as_i32(value).wrapping_add(as_i32(diff)))
        } else {
            value.wrapping_add(diff)
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
pub(crate) struct PredictorContextV2 {
    gps_home: GpsPosition,
    min_motor: u32,
    min_throttle: u32,
    motor_0: u32,
    vbat_reference: u32,
}
impl PredictorContextV2 {
    pub(crate) fn new() -> Self {
        // TODO
        Self {
            gps_home: GpsPosition {
                latitude: 0,
                longitude: 0,
            },
            min_motor: 0,
            min_throttle: 0,
            motor_0: 0,
            vbat_reference: 0,
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

    pub(crate) fn set_last(&mut self, last: Option<u32>) {
        self.last = last;
    }

    pub(crate) fn set_last_2(&mut self, last: Option<u32>, last_last: Option<u32>) {
        self.last = last;
        self.last_last = last_last;
    }
}

#[inline]
pub(crate) fn straight_line<T>(history: History<T>) -> T
where
    T: TemporaryOverflow + Default,
    T::Wide: Copy + Sub<Output = T::Wide> + Add<Output = T::Wide> + PartialOrd,
{
    match history {
        History::Two(last, last_last) => {
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
        History::One(last) => last,
        History::None => T::default(),
    }
}

#[inline]
fn average<T>(history: History<T>) -> T
where
    T: TemporaryOverflow + Copy + Default,
    T::Wide: Add<Output = T::Wide> + Div<Output = T::Wide> + From<u8>,
{
    match history {
        History::None => T::default(),
        History::One(last) => last,
        History::Two(last, last_last) => {
            T::truncate_from((last.widen() + last_last.widen()) / 2.into())
        }
    }
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
            #[expect(clippy::cast_possible_truncation)]
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

    use super::History;

    #[case(History::None => 0)]
    #[case(History::One(10) => 10)]
    #[case(History::One(-2) => -2)]
    #[case(History::Two(12, 10) => 14)]
    #[case(History::Two(10, 12) => 8)]
    #[case(History::Two(0, i8::MAX) => -i8::MAX)]
    #[case(History::Two(0, i8::MIN) => 0 ; "underflow")]
    #[case(History::Two(126, 0) => 126 ; "overflow")]
    fn straight_line_signed(history: History<i8>) -> i8 {
        super::straight_line(history)
    }

    #[case(History::Two(2, 2) => 2)]
    #[case(History::Two(12, 10) => 14)]
    #[case(History::Two(10, 12) => 8)]
    #[case(History::Two(0, u8::MIN) => 0 ; "underflow")]
    #[case(History::Two(u8::MAX - 1, 0) => 254 ; "overflow")]
    #[case(History::Two(0, u8::MAX) => 0 ; "negative result")]
    fn straight_line_unsigned(history: History<u8>) -> u8 {
        super::straight_line(history)
    }

    #[case(History::None => 0)]
    #[case(History::One(-1) => -1)]
    #[case(History::Two(2, -1) => 0)]
    #[case(History::Two(i32::MAX, 1) => 0x4000_0000 ; "overflow")]
    fn average_signed(history: History<i32>) -> i32 {
        super::average(history)
    }

    #[case(History::None => 0)]
    #[case(History::One(1) => 1)]
    #[case(History::Two(2, 10) => 6)]
    #[case(History::Two(u32::MAX, 1) => 0x8000_0000 ; "overflow")]
    fn average_unsigned(history: History<u32>) -> u32 {
        super::average(history)
    }
}
