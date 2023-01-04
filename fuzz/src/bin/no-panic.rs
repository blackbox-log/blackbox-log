#![no_main]

use blackbox_log::prelude::*;

blackbox_fuzz::fuzz_target!(|data: &[u8]| {
    let f = blackbox_log::File::new(data);
    for mut reader in f.iter() {
        let Ok(headers) = Headers::parse(&mut reader) else { return; };
        let mut data = DataParser::new(reader, &headers);

        while data.next().is_some() {}
    }
});
