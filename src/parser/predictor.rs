use num_enum::TryFromPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
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
    VBatRef,
    LastMainFrameTime,
    MinMotor,
    // HomeLon = 256,
}

impl Predictor {
    pub fn apply(self, value: i64 /*, current: i64, previous: i64, previous2: i64 */) -> i64 {
        let diff = match self {
            Self::Zero => 0,
            // Self::Previous => previous,
            // Self::StraightLine => (2 * previous) - previous2,
            // Self::Average2 => (previous + previous2) / 2,
            // Self::MinThrottle => todo!(),
            // Self::Motor0 => todo!(),
            // Self::Increment => todo!(),
            // Self::HomeLat => todo!(), // TODO: check that lat = 0, lon = 1
            Self::FifteenHundred => 1500,
            // Self::VBatRef => todo!(),
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
