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
    ) -> i64 {
        let last = last.unwrap_or(0);
        let last_last = last_last.unwrap_or(0);

        let diff = match self {
            Self::Zero => 0,
            Self::Previous => last,
            Self::StraightLine => (2 * last) - last_last,
            Self::Average2 => (last + last_last) / 2,
            // Self::MinThrottle => todo!(),
            // Self::Motor0 => todo!(),
            // Self::Increment => todo!(),
            // Self::HomeLat => todo!(), // TODO: check that lat = 0, lon = 1
            Self::FifteenHundred => 1500,
            Self::VBatReference => headers.vbat_reference.into(),
            // Self::LastMainFrameTime => todo!(),
            // Self::MinMotor => todo!(),
            // Self::HomeLon => todo!(),
            predictor => {
                tracing::warn!("found unimplemented predictor: {predictor:?}");
                0
            }
        };

        value + diff
    }
}
