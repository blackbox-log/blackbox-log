#![no_main]

use blackbox_fuzz::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let f = blackbox::File::new(data);

    for i in 0..f.log_count() {
        let _ = f.parse_by_index(i);
    }
});
