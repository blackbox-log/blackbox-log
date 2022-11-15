use std::io;
use std::io::Write;
use std::os::unix::io::AsRawFd;

pub use blackbox_log::parser::decode::no_error as decode;
use blackbox_log::parser::Reader;
use blackbox_sys::stream::Stream;
use libfuzzer_sys::arbitrary;
pub use libfuzzer_sys::arbitrary::Arbitrary;
pub use libfuzzer_sys::fuzz_target;
use memfile::MemFile;

#[derive(Debug, Arbitrary)]
pub struct AlignedBytes {
    bytes: Vec<u8>,
}

impl AlignedBytes {
    pub fn to_streams(&self) -> io::Result<(Stream, Reader)> {
        let bytes: &[u8] = &self.bytes;
        let mut f = MemFile::create_default("blackbox-sys-input")?;
        f.write_all(bytes)?;
        f.flush()?;

        Ok((Stream::new(f.as_raw_fd()), Reader::new(bytes)))
    }
}

pub fn float_eq(expected: f64, got: f64) {
    let threshold_absolute = 0.0001;
    let threshold_percent = 0.1;

    if expected.is_finite() && got.is_finite() {
        let diff = expected - got;

        if diff.abs() > threshold_absolute {
            let diff = diff / expected * 100.;
            assert!(
                diff.abs() <= threshold_percent,
                "{got} is differs from {expected} by {diff}% ({threshold_percent}% allowed)"
            );
        }
    } else {
        assert_eq!(expected.is_nan(), got.is_nan());
        assert_eq!(expected.is_infinite(), got.is_infinite());
        assert_eq!(expected.is_sign_positive(), got.is_sign_positive());
    }
}
