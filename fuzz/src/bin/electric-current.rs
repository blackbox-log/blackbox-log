#![no_main]

use blackbox_log::parser::headers::{CurrentMeterConfig, Headers};
use blackbox_log::units::{si, ElectricCurrent, FromRaw};

blackbox_fuzz::fuzz_target!(|input: (u16, i16, i16)| {
    let (raw, offset, scale) = input;

    if scale == 0 {
        return;
    }

    let mut headers = Headers::default();
    headers.current_meter = Some(CurrentMeterConfig { offset, scale });

    let mine = ElectricCurrent::from_raw(raw.into(), &headers);
    let mine = mine.get::<si::electric_current::milliampere>();

    let mut log = blackbox_sys::parser::FlightLog::new();
    log.sys_config_mut().currentMeterOffset = offset;
    log.sys_config_mut().currentMeterScale = scale;

    let expected = log.amperage_to_milliamps(raw);

    if expected > 0 {
        let diff = {
            let expected = f64::from(expected);
            (mine - expected).abs() / expected * 100.
        };

        let epsilon = 1.;
        assert!(
            diff <= epsilon,
            "{mine:.2} differs from {expected} by {diff}% (>{epsilon}%)"
        );
    }
});
