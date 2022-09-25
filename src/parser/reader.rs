use bitter::BitReader as _;
use std::io::{self, Read};

pub use bitter::BigEndianReader as BitReader;

pub struct Reader<'data> {
    index: usize,
    data: &'data [u8],
    bits: Option<BitReader<'data>>,
}

impl<'data> Reader<'data> {
    pub fn new(data: &'data [u8]) -> Self {
        if data.len() == usize::MAX {
            panic!("cannot create a Reader containing usize::MAX bytes");
        }

        Self {
            index: 0,
            data,
            bits: None,
        }
    }

    /// Leave bits mode, skipping any remaining bits if not byte aligned
    pub fn bytes<'reader>(&'reader mut self) -> ByteReader<'data, 'reader> {
        if let Some(bits) = self.bits.take() {
            // TODO: check how it handles partially read bytes
            self.index = self.data.len() - bits.bytes_remaining();
        }

        ByteReader(self)
    }

    pub fn bits<'reader: 'bits, 'bits>(&'reader mut self) -> &'bits mut BitReader<'data> {
        self.bits
            .get_or_insert_with(|| BitReader::new(&self.data[self.index..]))
    }

    #[must_use]
    pub fn is_byte_aligned(&self) -> bool {
        self.bits.as_ref().map_or(true, BitReader::byte_aligned)
    }

    pub fn byte_align(&mut self) {
        if let Some(ref mut bits) = self.bits {
            while !bits.byte_aligned() {
                bits.read_bit();
            }
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bits.as_ref().map_or(
            (self.data.len() - self.index) == 0,
            bitter::BitReader::is_empty,
        )
    }
}

pub struct ByteReader<'data: 'reader, 'reader>(&'reader mut Reader<'data>);

impl<'data, 'reader> ByteReader<'data, 'reader> {
    #[must_use]
    /// Counts the current byte if it has only been partially read
    fn remaining(&self) -> usize {
        self.0.data.len() - self.0.index
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn iter<'me>(&'me mut self) -> Bytes<'data, 'reader, 'me> {
        Bytes(self)
    }

    pub fn peek(&self) -> Option<u8> {
        self.0.data.get(self.0.index).copied()
    }

    pub fn read_line(&mut self) -> Option<&'data [u8]> {
        let start = dbg!(self.0.index);

        let rest = self.0.data.get(start..).filter(|x| !x.is_empty())?;

        if let Some(len) = rest.iter().position(|b| *b == b'\n') {
            self.0.index += len + 1; // Skip the '\n'

            let end = start + len;
            self.0.data.get(start..end)
        } else {
            self.0.index = self.0.data.len();
            self.0.data.get(start..)
        }
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        let byte = self.peek();
        if byte.is_some() {
            self.0.index += 1;
        }
        byte
    }

    pub fn read_i8(&mut self) -> Option<i8> {
        #[allow(clippy::cast_possible_wrap)]
        self.read_u8().map(|x| x as i8)
    }

    pub fn read_u24(&mut self) -> Option<u32> {
        if self.remaining() < 3 {
            return None;
        }

        let mut bytes = [0; 4];
        let slice = &mut bytes[0..3];

        let start = self.0.index;
        slice.copy_from_slice(&self.0.data[start..(start + 3)]);
        self.0.index += 3;

        Some(u32::from_le_bytes(bytes))
    }
}

macro_rules! impl_read {
    ($read:ident, $type:ty, $iread:ident, $itype:ty) => {
        pub fn $read(&mut self) -> Option<$type> {
            const BYTES: usize = (<$type>::BITS / 8) as usize;

            if self.remaining() < BYTES {
                return None;
            }

            let mut bytes = [0; BYTES];
            let start = self.0.index;
            bytes.copy_from_slice(&self.0.data[start..(start + BYTES)]);
            self.0.index += BYTES;

            Some(<$type>::from_le_bytes(bytes))
        }

        pub fn $iread(&mut self) -> Option<$itype> {
            self.$read().map(|x| x as $itype)
        }
    };
}

impl<'data, 'reader> ByteReader<'data, 'reader> {
    impl_read!(read_u16, u16, read_i16, i16);
    impl_read!(read_u32, u32, read_i32, i32);
}

impl<'data, 'reader> Read for ByteReader<'data, 'reader> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = buf.len().min(self.remaining());

        let start = self.0.index;
        let slice = &self.0.data[start..(start + len)];
        buf[0..len].copy_from_slice(slice);

        self.0.index += len;
        Ok(len)
    }
}

pub struct Bytes<'data: 'reader, 'reader: 'bytes, 'bytes>(&'bytes mut ByteReader<'data, 'reader>);

impl Iterator for Bytes<'_, '_, '_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.read_u8()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_align() {
        let mut reader = Reader::new(&[0, 1]);

        {
            let bits = reader.bits();
            bits.read_bit();
        }

        assert!(!reader.is_byte_aligned());
        reader.byte_align();
        assert!(reader.is_byte_aligned());

        assert_eq!(Some(1), reader.bytes().read_u8());
    }

    #[test]
    fn implicit_byte_align() {
        let mut reader = Reader::new(&[0, 1]);

        {
            let bits = reader.bits();
            bits.read_bit();
        }

        assert!(!reader.is_byte_aligned());
        assert_eq!(Some(1), reader.bytes().read_u8());
        assert!(reader.is_byte_aligned());

        assert_eq!(None, reader.bytes().read_u8());
    }

    #[test]
    fn bytes_read_line() {
        let mut reader = Reader::new(&[b'a', 0, b'\n', b'b']);
        let mut bytes = reader.bytes();

        assert_eq!(Some(b"a\0".as_ref()), bytes.read_line());
        assert_eq!(Some(b'b'), bytes.read_u8());
    }

    #[test]
    fn bytes_read_line_without_newline() {
        let mut reader = Reader::new(&[b'a', 0]);
        let mut bytes = reader.bytes();

        assert_eq!(Some(b"a\0".as_ref()), bytes.read_line());
        assert_eq!(None, bytes.read_u8());
    }

    #[test]
    fn bytes_read_line_empty() {
        let mut reader = Reader::new(&[]);
        let mut bytes = reader.bytes();

        assert_eq!(None, bytes.read_line());
    }

    #[test]
    fn bytes_read() {
        let input = [0, 1, 2, 3];

        let mut reader = Reader::new(&input);
        let mut bytes = reader.bytes();

        let mut buf = [0; 4];
        let read = bytes.read(&mut buf).unwrap();

        assert_eq!(read, input.len());
        assert_eq!(buf, input);
        assert!(bytes.is_empty());
    }

    #[test]
    fn bytes_read_exact() {
        let input = [0, 1, 2, 3];

        let mut reader = Reader::new(&input);
        let mut bytes = reader.bytes();

        let mut buf = [0; 4];
        bytes.read_exact(&mut buf).unwrap();

        assert_eq!(buf, input);
        assert!(bytes.is_empty());
    }

    #[test]
    fn bytes_read_empty() {
        let mut reader = Reader::new(&[]);
        let mut bytes = reader.bytes();

        let mut buf = [0; 1];
        let read = bytes.read(&mut buf).unwrap();

        assert_eq!([0], buf);
        assert_eq!(read, 0);
    }

    #[test]
    fn bytes_iter() {
        let mut reader = Reader::new(&[0]);
        let mut bytes = reader.bytes();
        let mut iter = bytes.iter();

        assert_eq!(Some(0), iter.next());
        assert_eq!(None, iter.next());
        assert!(bytes.is_empty());
    }
}
