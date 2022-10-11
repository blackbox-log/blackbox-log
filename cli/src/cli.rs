#![allow(clippy::default_trait_access)]

use std::ffi::OsString;
use std::fmt::{self, Display};
use std::path::PathBuf;

use blackbox::parser::{MainUnit, SlowUnit};
use blackbox::units::{Acceleration, Amperage, FlagSet, Rotation, Voltage};
use bpaf::{construct, Bpaf, FromOsStr, Parser};
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CliUnit {
    FrameTime(FrameTimeUnit),
    Amperage(AmperageUnit),
    VBat(VBatUnit),
    Acceleration(AccelerationUnit),
    Rotation(RotationUnit),
    Height(HeightUnit),
    GpsSpeed(GpsSpeedUnit),
    Flag(FlagUnit),
    Unitless,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CliUnitKind {
    FrameTime,
    Amperage,
    VBat,
    Acceleration,
    Rotation,
    Height,
    GpsSpeed,
    Flag,
    Unitless,
}

impl From<MainUnit> for CliUnitKind {
    fn from(val: MainUnit) -> Self {
        match val {
            MainUnit::FrameTime => CliUnitKind::FrameTime,
            MainUnit::Amperage => CliUnitKind::Amperage,
            MainUnit::Voltage => CliUnitKind::VBat,
            MainUnit::Acceleration => CliUnitKind::Acceleration,
            MainUnit::Rotation => CliUnitKind::Rotation,
            MainUnit::Unitless => CliUnitKind::Unitless,
        }
    }
}

impl From<SlowUnit> for CliUnitKind {
    fn from(val: SlowUnit) -> Self {
        match val {
            SlowUnit::FlightMode => CliUnitKind::Flag,
            SlowUnit::Unitless => CliUnitKind::Unitless,
        }
    }
}

impl CliUnit {
    pub fn is_raw(self) -> bool {
        match self {
            Self::FrameTime(_) | Self::Height(_) | Self::GpsSpeed(_) => false,
            Self::Amperage(a) => a == AmperageUnit::Raw,
            Self::VBat(v) => v == VBatUnit::Raw,
            Self::Acceleration(x) => x == AccelerationUnit::Raw,
            Self::Rotation(r) => r == RotationUnit::Raw,
            Self::Flag(f) => f == FlagUnit::Raw,
            Self::Unitless => true,
        }
    }
}

impl Display for CliUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::FrameTime(t) => t.fmt(f),
            Self::Amperage(a) => a.fmt(f),
            Self::VBat(v) => v.fmt(f),
            Self::Acceleration(a) => a.fmt(f),
            Self::Rotation(r) => r.fmt(f),
            Self::Height(h) => h.fmt(f),
            Self::GpsSpeed(s) => s.fmt(f),
            Self::Flag(flags) => flags.fmt(f),
            Self::Unitless => Ok(()),
        }
    }
}

macro_rules! from_os_str_impl {
    ($for:ident { $( $($s:literal)|+ => $value:expr, )+ _ => $err:literal $(,)? } $(,)?) => {
        impl FromOsStr for $for {
            type Out = Self;

            fn from_os_str(mut s: OsString) -> Result<Self::Out, String> {
                s.make_ascii_lowercase();

                $(
                    if $(s == $s)||+ {
                         Ok($value)
                    }
                )else+

                else { Err($err.to_owned()) }
            }
        }
    };
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AmperageUnit {
    Raw,
    Milliamps,
    #[default]
    Amps,
}

impl AmperageUnit {
    pub fn format(&self, amps: Amperage) -> String {
        match self {
            Self::Raw => amps.as_raw().to_string(),
            Self::Milliamps => format!("{:.0}", amps.as_milliamps()),
            Self::Amps => format!("{:.3}", amps.as_milliamps() / 1000.),
        }
    }
}

from_os_str_impl!(AmperageUnit {
    "raw" => Self::Raw,
    "ma" => Self::Milliamps,
    "a" => Self::Amps,
    _ => "expected raw, mA, or A",
});

impl fmt::Display for AmperageUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Raw => "raw",
            Self::Milliamps => "mA",
            Self::Amps => "A",
        };

        f.write_str(s)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FrameTimeUnit {
    #[default]
    Microseconds,
    Milliseconds,
    Seconds,
}

impl FrameTimeUnit {
    pub fn format(&self, us: u64) -> String {
        match self {
            Self::Microseconds => us.to_string(),
            Self::Milliseconds => format_decimal::<1000>(us),
            Self::Seconds => format_decimal::<1_000_000>(us),
        }
    }
}

from_os_str_impl!(FrameTimeUnit {
    "us" | "micros" => Self::Microseconds,
    "ms" | "millis" => Self::Milliseconds,
    "s"| "sec" => Self::Seconds,
    _ => "expected us, ms, or s",
});

impl fmt::Display for FrameTimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Microseconds => "us",
            Self::Milliseconds => "ms",
            Self::Seconds => "s",
        };

        f.write_str(s)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum HeightUnit {
    #[default]
    Centimeters,
    Meters,
    Feet,
}

from_os_str_impl!(HeightUnit {
    "cm" => Self::Centimeters,
    "m" => Self::Meters,
    "ft" => Self::Feet,
    _ => "expected cm, m, or ft",
});

impl fmt::Display for HeightUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Centimeters => "cm",
            Self::Meters => "m",
            Self::Feet => "ft",
        };

        f.write_str(s)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RotationUnit {
    #[default]
    Raw,
    /// Degrees/second
    Degrees,
    /// Radians/second
    Radians,
}

impl RotationUnit {
    pub fn format(&self, rotation: Rotation) -> String {
        match self {
            Self::Raw => rotation.as_raw().to_string(),
            Self::Degrees => format!("{:.2}", rotation.as_degrees()),
            Self::Radians => format!("{:.2}", rotation.as_radians()),
        }
    }
}

from_os_str_impl!(RotationUnit {
    "raw" => Self::Raw,
    "deg/s" | "deg" => Self::Degrees,
    "rad/s" | "rad" => Self::Radians,
    _ => "expected raw, deg/s, or rad/s",
});

impl fmt::Display for RotationUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Raw => "raw",
            Self::Degrees => "deg/s",
            Self::Radians => "rad/s",
        };

        f.write_str(s)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AccelerationUnit {
    #[default]
    Raw,
    Gs,
    /// Meters per second squared
    Mps2,
}

impl AccelerationUnit {
    pub fn format(&self, accel: Acceleration) -> String {
        match self {
            Self::Raw => accel.as_raw().to_string(),
            Self::Gs => format!("{:.3}", accel.as_gs()),
            Self::Mps2 => format!("{:.3}", accel.as_meters_per_sec_sq()),
        }
    }
}

from_os_str_impl!(AccelerationUnit {
    "raw" => Self::Raw,
    "g" => Self::Gs,
    "m/s2" | "m/s/s" | "mps2" => Self::Mps2,
    _ => "expected raw, g, or m/s/s",
});

impl fmt::Display for AccelerationUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Raw => "raw",
            Self::Gs => "g",
            Self::Mps2 => "m/s/s",
        };

        f.write_str(s)
    }
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GpsSpeedUnit {
    #[default]
    /// Meters per second
    Mps,

    /// Kilometers per hour
    Kph,

    /// Miles per hour
    Mph,
}

from_os_str_impl!(GpsSpeedUnit {
    "m/s" | "mps" => Self::Mps,
    "kph" | "k/h" => Self::Kph,
    "mph" => Self::Mph,
    _ => "expected m/s, kph, or mph",
});

impl fmt::Display for GpsSpeedUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Mps => "mps",
            Self::Kph => "kph",
            Self::Mph => "mph",
        };

        f.write_str(s)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum VBatUnit {
    Raw,
    Millivolts,
    #[default]
    Volts,
}

impl VBatUnit {
    pub fn format(&self, volts: Voltage) -> String {
        match self {
            Self::Raw => volts.as_raw().to_string(),
            Self::Millivolts => format!("{:.0}", volts.as_millivolts()),
            Self::Volts => format!("{:.3}", volts.as_millivolts() / 1000.),
        }
    }
}

from_os_str_impl!(VBatUnit {
    "raw" => Self::Raw,
    "mv" => Self::Millivolts,
    "v" => Self::Volts,
    _ => "expected raw, mV, or V",
});

impl fmt::Display for VBatUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Raw => "raw",
            Self::Millivolts => "mV",
            Self::Volts => "V",
        };

        f.write_str(s)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FlagUnit {
    Raw,
    #[default]
    Flags,
}

impl FlagUnit {
    pub fn format<F: FlagSet>(&self, flags: F) -> String {
        match self {
            Self::Raw => flags.as_raw().to_string(),
            Self::Flags => {
                let flags = flags.as_names();
                if flags.is_empty() {
                    "0".to_owned()
                } else {
                    flags
                        .into_iter()
                        .enumerate()
                        .map(|(i, name)| {
                            if i == 0 {
                                name.to_owned()
                            } else {
                                format!("|{name}")
                            }
                        })
                        .collect()
                }
            }
        }
    }
}

from_os_str_impl!(FlagUnit {
    "raw" => Self::Raw,
    "flags" | "flag" => Self::Flags,
    _ => "expected raw or flags",
});

impl fmt::Display for FlagUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Raw => "raw",
            Self::Flags => "flags",
        };

        f.write_str(s)
    }
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, private, version)]
#[allow(unused, clippy::default_trait_access)]
pub(crate) struct Cli {
    #[bpaf(external)]
    pub index: Vec<usize>,

    /// Prints the limits and range of each field
    pub limits: bool,

    /// Writes log to stdout instead of a file
    pub stdout: bool,

    #[bpaf(argument("unit"), fallback(Default::default()))]
    pub unit_amperage: AmperageUnit,

    #[bpaf(argument("unit"), fallback(Default::default()))]
    pub unit_flags: FlagUnit,

    #[bpaf(argument("unit"), fallback(Default::default()))]
    pub unit_frame_time: FrameTimeUnit,

    #[bpaf(argument("unit"), fallback(Default::default()))]
    pub unit_height: HeightUnit,

    #[bpaf(argument("unit"), fallback(Default::default()))]
    pub unit_rotation: RotationUnit,

    #[bpaf(argument("unit"), fallback(Default::default()))]
    pub unit_acceleration: AccelerationUnit,

    #[bpaf(argument("unit"), fallback(Default::default()))]
    pub unit_gps_speed: GpsSpeedUnit,

    #[bpaf(argument("unit"), fallback(Default::default()))]
    pub unit_vbat: VBatUnit,

    #[bpaf(external)]
    pub altitude_offset: i16,

    /// Merges GPS data into the main CSV file instead of writing it
    /// separately
    pub merge_gps: bool,

    #[bpaf(external)]
    pub current_meter: Option<CurrentMeter>,

    // TODO: alias: simulate-imu
    #[bpaf(external)]
    pub imu: Option<Imu>,

    // TODO
    // #[arg(long)]
    // /// Set magnetic declination in degrees.minutes format (e.g. -12.58 for New York)
    // declination: (),
    //
    // TODO
    // #[arg(long)]
    // /// Set magnetic declination in decimal degrees (e.g. -12.97 for New York)
    // declination_dec: (),
    #[bpaf(external)]
    pub verbosity: LevelFilter,

    // TODO: accept - for stdin
    // TODO: complete file paths
    /// One or more logs to parse
    #[bpaf(
        positional("file"),
        guard(at_least_one, "at least one log file is required")
    )]
    pub logs: Vec<PathBuf>,
}

fn index() -> impl Parser<Vec<usize>> {
    let help = "Chooses which log(s) should be decoded or omit to decode all\n(applies to all \
                logs & can be repeated)";

    bpaf::short('i')
        .long("index")
        .help(help)
        .argument::<usize>("index")
        .many()
        .map(|mut v| {
            v.sort_unstable();
            v.dedup();
            v
        })
}

fn at_least_one(logs: &Vec<PathBuf>) -> bool {
    !logs.is_empty()
}

fn altitude_offset() -> impl Parser<i16> {
    let old = bpaf::long("alt-offset").argument::<i16>("").hide();
    let new = bpaf::long("altitude-offset")
        .help("Sets the altitude offset in meters")
        .argument::<i16>("offset");

    construct!([new, old]).fallback(0)
}

#[derive(Debug, Clone, Default)]
#[allow(unused)]
pub struct CurrentMeter {
    scale: Option<i16>,
    offset: Option<i16>,
}

fn current_meter() -> impl Parser<Option<CurrentMeter>> {
    let meter = {
        let help = "Simulates a virtual current meter using throttle data";

        let old = bpaf::long("simulate-current-meter").switch().hide();
        let new = bpaf::long("current-meter").help(help).switch();

        construct!([new, old])
    };

    let scale = {
        let help =
            "Overrides the current meter scale for the simulation\n(implies --current-meter)";

        let old = bpaf::long("sim-current-meter-scale")
            .argument::<i16>("")
            .hide();
        let new = bpaf::long("current-scale")
            .help(help)
            .argument::<i16>("scale");

        construct!([new, old]).map(Some).fallback(None)
    };

    let offset = {
        let help =
            "Overrides the current meter offset for the simulation\n(implies --current-meter)";

        let old = bpaf::long("sim-current-meter-offset")
            .argument::<i16>("")
            .hide();
        let new = bpaf::long("current-offset")
            .help(help)
            .argument::<i16>("offset");

        construct!([new, old]).map(Some).fallback(None)
    };

    construct!(meter, scale, offset).map(|(meter, scale, offset)| {
        (meter || scale.is_some() || offset.is_some()).then_some(CurrentMeter { scale, offset })
    })
}

#[derive(Debug, Clone, Default)]
#[allow(unused)]
pub struct Imu {
    deg_in_names: bool,
    ignore_mag: bool,
}

fn imu() -> impl Parser<Option<Imu>> {
    let help = "Computes tilt, roll, and heading information from gyro,\naccelerometer, and \
                magnetometer data";

    let imu = {
        let old = bpaf::long("simulate-imu").switch().hide();
        let new = bpaf::long("imu").help(help).switch();

        construct!([new, old])
    };

    let deg = {
        let old = bpaf::long("include-imu-degrees").switch().hide();
        let new = bpaf::long("imu-deg")
            .help("Includes (deg) in the tilt/roll/heading header (implies --imu)")
            .switch();

        construct!([new, old])
    };

    let ignore_mag = bpaf::long("imu-ignore-mag")
        .help("Ignores magnetometer when computing heading (implies --imu)")
        .switch();

    construct!(imu, deg, ignore_mag).map(|(imu, deg, ignore_mag)| {
        (imu || deg || ignore_mag).then_some(Imu {
            deg_in_names: deg,
            ignore_mag,
        })
    })
}

fn verbosity() -> impl Parser<LevelFilter> {
    const DEFAULT: usize = if cfg!(debug_assertions) { 4 } else { 3 };
    const LEVELS: [LevelFilter; 6] = [
        LevelFilter::OFF,
        LevelFilter::ERROR,
        LevelFilter::WARN,
        LevelFilter::INFO,
        LevelFilter::DEBUG,
        LevelFilter::TRACE,
    ];

    fn plural(x: usize) -> &'static str {
        if x == 1 { "" } else { "s" }
    }

    let debug = bpaf::long("debug").switch().hide();

    let max = DEFAULT;
    let help = format!("Reduces debug output up to {max} time{}", plural(max));
    let quiet = bpaf::short('q')
        .long("quiet")
        .help(help)
        .req_flag(())
        .many()
        .map(|v| v.len());

    let max = LEVELS.len() - DEFAULT - 1;
    let help = format!("Increases debug output up to {max} time{}", plural(max));
    let verbose = bpaf::short('v')
        .long("verbose")
        .help(help)
        .req_flag(())
        .many()
        .map(|v| v.len());

    construct!(debug, verbose, quiet).map(|(debug, v, q)| {
        if debug {
            LEVELS[LEVELS.len() - 1]
        } else {
            LEVELS[(v + DEFAULT).saturating_sub(q).min(LEVELS.len() - 1)]
        }
    })
}

impl Cli {
    pub fn parse() -> Self {
        cli()
            .usage("Usage: blackbox_decode [options] <log>...")
            .run()
    }

    pub fn get_unit(&self, unit: CliUnitKind) -> CliUnit {
        match unit {
            CliUnitKind::FrameTime => CliUnit::FrameTime(self.unit_frame_time),
            CliUnitKind::Amperage => CliUnit::Amperage(self.unit_amperage),
            CliUnitKind::VBat => CliUnit::VBat(self.unit_vbat),
            CliUnitKind::Acceleration => CliUnit::Acceleration(self.unit_acceleration),
            CliUnitKind::Rotation => CliUnit::Rotation(self.unit_rotation),
            CliUnitKind::Height => CliUnit::Height(self.unit_height),
            CliUnitKind::GpsSpeed => CliUnit::GpsSpeed(self.unit_gps_speed),
            CliUnitKind::Flag => CliUnit::Flag(self.unit_flags),
            CliUnitKind::Unitless => CliUnit::Unitless,
        }
    }
}

fn format_decimal<const DIVISOR: u64>(x: u64) -> String {
    // let DIVISOR = 1_000_000;
    let mut s = String::new();
    s.push_str(&(x / DIVISOR).to_string());
    s.push('.');
    s.push_str(&(x % DIVISOR).to_string());
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bpaf_invariants() {
        cli().check_invariants(true);
    }
}
