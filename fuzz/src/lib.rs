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
