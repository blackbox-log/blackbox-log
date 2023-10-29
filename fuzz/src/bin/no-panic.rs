#![no_main]

fuzz::fuzz_target!(|data: &[u8]| {
    let f = blackbox_log::File::new(data);
    for headers in f.iter().filter_map(Result::ok) {
        let mut data = headers.data_parser();
        while data.next().is_some() {}
    }
});
