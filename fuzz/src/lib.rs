pub use bitter::{BigEndianReader, BitReader};
pub use blackbox::encoding;
pub use libfuzzer_sys::arbitrary::Arbitrary;
pub use libfuzzer_sys::fuzz_target;

use libfuzzer_sys::arbitrary;
use memfile::MemFile;
use reference_impl::stream::Stream;
use std::io;
use std::io::Write;
use std::os::unix::io::AsRawFd;

#[derive(Debug)]
pub struct UnalignedBytes {
    offset: u8,
    bytes: Vec<u8>,
}

impl<'a> Arbitrary<'a> for UnalignedBytes {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let bytes: Vec<u8> = u.arbitrary()?;
        let offset = if bytes.is_empty() {
            0
        } else {
            u.choose_index(8)?.try_into().unwrap()
        };

        Ok(Self { offset, bytes })
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (1, None)
    }
}

impl UnalignedBytes {
    pub fn to_streams_unaligned(&self) -> io::Result<(Stream, BigEndianReader)> {
        let (mut reference, mut bitter) = self.to_streams_aligned()?;

        let offset = self.offset % 8;
        if offset > 0 {
            let reference_bits = reference.read_bits(offset);
            let bitter_bits = bitter.read_bits(offset.into()).unwrap_or(0);
            assert_eq!(u64::from(reference_bits), bitter_bits);
        }

        Ok((reference, bitter))
    }

    pub fn to_streams_aligned(&self) -> io::Result<(Stream, BigEndianReader)> {
        let mut f = MemFile::create_default("reference-impl-input")?;
        f.write_all(&self.bytes)?;
        f.flush()?;

        let reference = Stream::new(f.as_raw_fd());
        let bitter = BigEndianReader::new(self.bytes.as_slice());

        Ok((reference, bitter))
    }
}
