use num_enum::TryFromPrimitive;

use super::Headers;

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
        last: Option<i64>,
        last_last: Option<i64>,
        skipped_frames: i64,
    ) -> i64 {
        let diff = match self {
            Self::Zero => 0,
            Self::Previous => last.unwrap_or(0),
            Self::StraightLine => match (last, last_last) {
                (Some(last), Some(last_last)) => (last - last_last) + last,
                (Some(last), None) => last,
                _ => 0,
            },
            Self::Average2 => (last.unwrap_or(0) + last_last.unwrap_or(0)) / 2,
            // Self::MinThrottle => todo!(),
            // Self::Motor0 => todo!(),
            Self::Increment => 1 + skipped_frames + last.unwrap_or(0),
            // Self::HomeLat => todo!(), // TODO: check that lat = 0, lon = 1
            Self::FifteenHundred => 1500,
            Self::VBatReference => headers.vbat_reference.into(),
            // Self::LastMainFrameTime => todo!(),
            // Self::MinMotor => todo!(),
            // Self::HomeLon => todo!(),
            Self::MinThrottle
            | Self::Motor0
            | Self::HomeLat
            | Self::LastMainFrameTime
            | Self::MinMotor => {
                tracing::warn!("found unimplemented predictor: {self:?}");
                0
            }
        };

        value + diff
    }
}
