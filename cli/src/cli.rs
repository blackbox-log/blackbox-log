use blackbox::parser::Config;
use clap::{ArgAction, Parser, ValueEnum, ValueHint};
use std::path::PathBuf;
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AmperageUnit {
    #[default]
    Raw,
    #[value(name = "mA", alias = "ma")]
    Milliamps,
    #[value(name = "A", alias = "a")]
    Amps,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FrameTime {
    #[default]
    #[value(name = "us", alias = "micros")]
    Microseconds,
    #[value(name = "s")]
    Seconds,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum HeightUnit {
    #[default]
    #[value(name = "cm")]
    Centimeters,
    #[value(name = "m")]
    Meters,
    #[value(name = "ft")]
    Feet,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RotationUnit {
    #[default]
    Raw,

    #[value(name = "deg/s", alias = "deg")]
    /// Degrees/second
    Degrees,

    #[value(name = "rad/s", alias = "rad")]
    /// Radians/second
    Radians,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AccelerationUnit {
    #[default]
    Raw,

    #[value(name = "g")]
    Gs,

    #[value(name = "m/s2")]
    /// Meters per second squared
    Mps2,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum GpsSpeedUnit {
    #[default]
    /// Meters per second
    Mps,

    /// Kilometers per hour
    Kph,

    /// Miles per hour
    Mph,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum VBatUnit {
    #[default]
    Raw,

    #[value(name = "mV", alias = "mv")]
    Millivolts,

    #[value(name = "V", alias = "v")]
    Volts,
}

#[derive(Debug, Parser)]
#[command(about, author, version)]
pub(crate) struct Cli {
    // TODO: accept - for stdin
    #[arg(required(true), value_name = "log", value_hint = ValueHint::FilePath)]
    /// One or more logs to parse
    pub logs: Vec<PathBuf>,

    // #[arg(short, long, action = ArgAction::Help)]
    // /// Print this help message
    // help: bool,
    #[arg(short, long, value_name = "index")]
    /// Choose which log from the file should be decoded or omit to decode all
    pub index: Vec<usize>,

    #[arg(long)]
    /// Print the limits and range of each field
    pub limits: bool,

    #[arg(long)]
    /// Write log to stdout instead of a file
    pub stdout: bool,

    #[arg(long, value_enum, default_value_t, value_name = "unit")]
    pub unit_amperage: AmperageUnit,

    // TODO: --unit-frame-time
    #[arg(long, value_enum, default_value_t, value_name = "unit")]
    pub unit_height: HeightUnit,

    #[arg(long, value_enum, default_value_t, value_name = "unit")]
    pub unit_rotation: RotationUnit,

    #[arg(long, value_enum, default_value_t, value_name = "unit")]
    pub unit_acceleration: AccelerationUnit,

    #[arg(long, value_enum, default_value_t, value_name = "unit")]
    pub unit_gps_speed: GpsSpeedUnit,

    #[arg(long, value_enum, default_value_t, value_name = "unit")]
    pub unit_vbat: VBatUnit,

    #[arg(long, default_value_t, value_name = "offset", alias = "alt-offset")]
    /// Altitude offset in meters
    pub altitude_offset: u16,

    #[arg(long)]
    /// Merge GPS data into the main CSV file instead of writing it separately
    pub merge_gps: bool,

    #[arg(long, alias = "simulate-current-meter")]
    /// Simulate a virtual current meter using throttle data
    pub sim_current_meter: bool,

    #[arg(
        long,
        requires = "sim_current_meter",
        alias = "sim-current-meter-scale",
        alias = "simulate-current-meter-scale"
    )]
    /// Override the flight controller's current scale when simulating the current meter
    pub current_scale: bool,

    #[arg(
        long,
        requires = "sim_current_meter",
        alias = "sim-current-meter-offset",
        alias = "simulate-current-meter-offset"
    )]
    /// Override the flight controller's current offset when simulating the current meter
    pub current_offset: bool,

    #[arg(long, alias = "simulate-imu")]
    /// Compute tilt, roll, and heading information from gyro, accelerometer, and magnetometer data
    pub sim_imu: bool,

    #[arg(
        long,
        requires = "sim_imu",
        alias = "include-imu-deg",
        alias = "include-imu-degrees"
    )]
    /// Include (deg) in the tilt/roll/heading header
    pub imu_deg: bool,

    #[arg(long, requires = "sim_imu", alias = "imu-ignore-mag")]
    /// Ignore magnetometer when computing heading
    pub ignore_mag: bool,

    // TODO
    // #[arg(long)]
    // /// Set magnetic declination in degrees.minutes format (e.g. -12.58 for New York)
    // declination: (),
    //
    // TODO
    // #[arg(long)]
    // /// Set magnetic declination in decimal degrees (e.g. -12.97 for New York)
    // declination_dec: (),
    #[arg(long)]
    /// Show raw field values without applying predictions
    pub raw: bool,

    #[arg(short, long, action = ArgAction::Count, conflicts_with = "verbose")]
    /// Reduce output
    pub quiet: u8,

    #[arg(short, long, action = ArgAction::Count, alias = "debug")]
    /// Increase output
    pub verbose: u8,
}

impl Cli {
    pub fn log_level_filter(&self) -> LevelFilter {
        let default: u8 = if cfg!(debug_assertions) { 4 } else { 3 };
        match default.saturating_sub(self.quiet) + self.verbose {
            0 => LevelFilter::OFF,
            1 => LevelFilter::ERROR,
            2 => LevelFilter::WARN,
            3 => LevelFilter::INFO,
            4 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    }

    pub fn to_blackbox_config(&self) -> Config {
        Config { raw: self.raw }
    }
}
