pub use biterator::Biterator;
pub use blackbox::encoding;
pub use libfuzzer_sys::fuzz_target;

use memfile::MemFile;
use reference_impl::stream::Stream;
use std::io;
use std::io::Write;
use std::os::unix::io::AsRawFd;

pub fn get_streams(bytes: &[u8]) -> io::Result<(Stream, Biterator<&[u8]>)> {
    let mut f = MemFile::create_default("reference-impl-input")?;
    f.write_all(bytes)?;
    f.flush()?;

    let reference = Stream::new(f.as_raw_fd());
    let biterator = Biterator::new(bytes);

    Ok((reference, biterator))
}
