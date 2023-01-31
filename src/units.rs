use alloc::vec::Vec;

pub use uom::si;
pub use uom::si::f64::{
    Acceleration, AngularVelocity, ElectricCurrent, ElectricPotential, Length, Time, Velocity,
};

use crate::Headers;

#[allow(unreachable_pub)]
pub(crate) mod prelude {
    pub use super::si::acceleration::meter_per_second_squared as mps2;
    pub use super::si::angular_velocity::degree_per_second;
    pub use super::si::electric_current::{ampere, milliampere};
    pub use super::si::electric_potential::{millivolt, volt};
    pub use super::si::length::meter;
    pub use super::si::time::{microsecond, second};
    pub use super::si::velocity::meter_per_second;
    pub use super::{
        Acceleration, AngularVelocity, ElectricCurrent, ElectricPotential, Length, Time, Velocity,
    };
}

mod from_raw {
    #[allow(unreachable_pub)]
    pub trait FromRaw {
        type Raw;
        fn from_raw(raw: Self::Raw, headers: &super::Headers) -> Self;
    }
}

include_generated!("failsafe_phase");
include_generated!("flight_mode");
include_generated!("state");

pub(crate) use from_raw::FromRaw;

impl FromRaw for Time {
    type Raw = u64;

    fn from_raw(raw: Self::Raw, _headers: &Headers) -> Self {
        Self::new::<prelude::microsecond>(raw as f64)
    }
}

impl FromRaw for Acceleration {
    type Raw = i32;

    fn from_raw(raw: Self::Raw, headers: &Headers) -> Self {
        // TODO: switch to `standard_gravity` instead of `mps2` once
        // https://github.com/iliekturtles/uom/pull/351 lands

        let gs = f64::from(raw) / f64::from(headers.acceleration_1g.unwrap());
        Self::new::<prelude::mps2>(gs * 9.80665)
    }
}

impl FromRaw for AngularVelocity {
    type Raw = i32;

    fn from_raw(raw: Self::Raw, headers: &Headers) -> Self {
        let scale = headers.gyro_scale.unwrap();
        let rad = f64::from(scale) * f64::from(raw);

        AngularVelocity::new::<si::angular_velocity::radian_per_second>(rad)
    }
}

impl FromRaw for ElectricCurrent {
    type Raw = i32;

    fn from_raw(raw: Self::Raw, _headers: &Headers) -> Self {
        new_amps(raw)
    }
}

/// Correct from BF 3.1.7 (3.1.0?), INAV 2.0.0
#[inline(always)]
fn new_amps(raw: i32) -> ElectricCurrent {
    ElectricCurrent::new::<si::electric_current::centiampere>(raw.into())
}

impl FromRaw for ElectricPotential {
    type Raw = u32;

    fn from_raw(raw: Self::Raw, _headers: &Headers) -> Self {
        new_vbat(raw)
    }
}

/// Correct from BF 4.0.0, INAV 3.0.0?
#[inline(always)]
fn new_vbat(raw: u32) -> ElectricPotential {
    ElectricPotential::new::<si::electric_potential::centivolt>(raw.into())
}

impl FromRaw for Velocity {
    type Raw = u32;

    fn from_raw(raw: Self::Raw, _headers: &Headers) -> Self {
        Self::new::<si::velocity::centimeter_per_second>(raw.into())
    }
}

pub trait FlagSet {
    type Flag: Flag;

    /// Checks if a given flag is enabled.
    fn is_set(&self, flag: Self::Flag) -> bool;

    /// Returns the names of all enabled flags.
    fn as_names(&self) -> Vec<&'static str>;
}

pub trait Flag {
    /// Returns the name of this flag.
    fn as_name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! float_eq {
        ($left:expr, $right:expr) => {
            let epsilon = 0.0001;
            let diff = ($left - $right).abs();
            assert!(
                diff < epsilon,
                "{left} and {right} are greater than {epsilon} apart: {diff}",
                left = $left,
                right = $right
            );
        };
    }

    #[test]
    fn electric_current() {
        float_eq!(1.39, new_amps(139).get::<prelude::ampere>());
    }

    #[test]
    fn electric_potential() {
        float_eq!(16.32, new_vbat(1632).get::<prelude::volt>());
    }

    mod resolution {
        use super::*;

        #[test]
        fn time() {
            use si::time::{day, microsecond};

            let ms = Time::new::<microsecond>(1.);
            float_eq!(1., ms.get::<microsecond>());

            let d = Time::new::<day>(1.);
            float_eq!(1., d.get::<day>());

            float_eq!(
                ms.get::<microsecond>() + d.get::<microsecond>(),
                (ms + d).get::<microsecond>()
            );
        }

        #[test]
        fn acceleration() {
            use si::acceleration::{
                kilometer_per_second_squared as kmps2, millimeter_per_second_squared as mmps2,
            };

            let mm = Acceleration::new::<mmps2>(1.);
            float_eq!(1., mm.get::<mmps2>());

            let km = Acceleration::new::<kmps2>(1.);
            float_eq!(1., km.get::<kmps2>());

            float_eq!(
                mm.get::<mmps2>() + km.get::<mmps2>(),
                (mm + km).get::<mmps2>()
            );
        }

        #[test]
        fn angular_velocity() {
            use si::angular_velocity::degree_per_second as dps;

            let slow = AngularVelocity::new::<dps>(0.01);
            float_eq!(0.01, slow.get::<dps>());

            let fast = AngularVelocity::new::<dps>(5_000.);
            float_eq!(5_000., fast.get::<dps>());

            float_eq!(5_000.01, (slow + fast).get::<dps>());
        }

        #[test]
        fn electric_current() {
            use si::electric_current::{kiloampere, milliampere};

            let ma = ElectricCurrent::new::<milliampere>(1.);
            float_eq!(1., ma.get::<milliampere>());

            let ka = ElectricCurrent::new::<kiloampere>(1.);
            float_eq!(1., ka.get::<kiloampere>());

            float_eq!(
                ma.get::<milliampere>() + ka.get::<milliampere>(),
                (ma + ka).get::<milliampere>()
            );
        }

        #[test]
        fn electric_potential() {
            use si::electric_potential::{kilovolt, millivolt};

            let mv = ElectricPotential::new::<millivolt>(1.);
            float_eq!(1., mv.get::<millivolt>());

            let kv = ElectricPotential::new::<kilovolt>(1.);
            float_eq!(1., kv.get::<kilovolt>());

            float_eq!(
                mv.get::<millivolt>() + kv.get::<millivolt>(),
                (mv + kv).get::<millivolt>()
            );
        }
    }
}
