#![no_main]

blackbox_fuzz::fuzz_target!(|data: &[u8]| {
    let f = blackbox_log::File::new(data);
    for _ in f.parse_iter() {}
});
