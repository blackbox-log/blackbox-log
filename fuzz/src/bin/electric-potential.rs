#![no_main]

use blackbox_log::parser::headers::{Headers, VbatConfig};
use blackbox_log::units::{si, ElectricPotential, FromRaw};

blackbox_fuzz::fuzz_target!(|input: (u16, u8)| {
    let (raw, scale) = input;
    let mut headers = Headers::default();
    headers.vbat = Some(VbatConfig {
        reference: 0,
        scale,
    });

    let mine = ElectricPotential::from_raw(raw.into(), &headers);
    let mine = mine.get::<si::electric_potential::millivolt>();

    let mut log = blackbox_sys::parser::FlightLog::new();
    log.sys_config_mut().vbatscale = scale;
    let expected = log.vbat_to_millivolts(raw);

    #[allow(clippy::cast_possible_truncation)]
    if expected > 0 {
        let diff = i128::from(expected).abs_diff(mine.round() as i128);
        assert!(
            diff <= 1,
            "{expected} and {mine:.2} are different by {diff}"
        );
    }
});
