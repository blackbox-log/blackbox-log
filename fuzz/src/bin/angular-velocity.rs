#![no_main]

use blackbox_log::parser::headers::Headers;
use blackbox_log::units::{self, si, FromRaw};

blackbox_fuzz::fuzz_target!(|input: (i32, f32)| {
    let (raw, scale_rad) = input;

    let mut headers = Headers::default();
    headers.gyro_scale = Some(scale_rad);

    let mine = units::AngularVelocity::from_raw(raw, &headers);
    let mine = mine.get::<si::angular_velocity::radian_per_second>();

    let mut log = blackbox_sys::parser::FlightLog::new();
    log.sys_config_mut().gyroScale = scale_rad / 1_000_000.; // Uses rad/us, not rad/sec
    let expected = log.gyro_to_rad_per_sec(raw);

    blackbox_fuzz::float_eq(expected, mine);
});
