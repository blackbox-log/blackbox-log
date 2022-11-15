#![no_main]

use blackbox_log::parser::headers::Headers;
use blackbox_log::units::{self, si, FromRaw};

blackbox_fuzz::fuzz_target!(|input: (i32, u16)| {
    let (raw, one_g) = input;
    let mut headers = Headers::default();
    headers.acceleration_1g = Some(one_g);

    let mine = units::Acceleration::from_raw(raw, &headers);
    let mine = mine.get::<si::acceleration::meter_per_second_squared>();

    let mut log = blackbox_sys::parser::FlightLog::new();
    log.sys_config_mut().acc_1G = one_g;
    let expected = log.accel_to_gs(raw) as f64 * 9.80665;

    blackbox_fuzz::float_eq(expected, mine);
});
