use core::fmt;
#[cfg(feature = "std")]
use std::io::{self, Read};

#[derive(Clone)]
pub struct Reader<'data> {
    index: usize,
    data: &'data [u8],
}

#[derive(Debug, Clone)]
pub struct RestorePoint(usize);

impl<'data> Reader<'data> {
    #[must_use]
    pub fn new(data: &'data [u8]) -> Self {
        if data.len() == usize::MAX {
            panic!("cannot create a Reader containing usize::MAX bytes");
        }

        Self { index: 0, data }
    }

    pub fn get_restore_point(&self) -> RestorePoint {
        RestorePoint(self.index)
    }

    pub fn restore(&mut self, restore: RestorePoint) {
        self.index = restore.0;
    }

    /// Advances past all bytes not matching any of the needles, returning
    /// `true` if any are found.
    pub fn skip_until_any(&mut self, needles: &[u8]) -> bool {
        debug_assert_ne!(
            needles.len(),
            0,
            "searching for any of 0 bytes makes no sense"
        );

        let position = self.data[self.index..]
            .iter()
            .position(|x| needles.contains(x));

        if let Some(position) = position {
            self.index += position;
        }

        position.is_some()
    }

    #[must_use]
    /// Counts the current byte if it has only been partially read
    fn remaining(&self) -> usize {
        self.data.len() - self.index
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn iter<'me>(&'me mut self) -> Bytes<'data, 'me> {
        Bytes(self)
    }

    pub fn peek(&self) -> Option<u8> {
        self.data.get(self.index).copied()
    }

    pub fn read_line(&mut self) -> Option<&'data [u8]> {
        let start = self.index;

        let rest = self.data.get(start..).filter(|x| !x.is_empty())?;

        if let Some(len) = rest.iter().position(|b| *b == b'\n') {
            self.index += len + 1; // Skip the '\n'

            let end = start + len;
            self.data.get(start..end)
        } else {
            self.index = self.data.len();
            self.data.get(start..)
        }
    }

    pub fn read_n_bytes(&mut self, n: usize) -> &'data [u8] {
        let len = n.min(self.remaining());

        let start = self.index;
        let slice = &self.data[start..(start + len)];

        self.index += len;
        slice
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        let byte = self.peek();
        if byte.is_some() {
            self.index += 1;
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

        let start = self.index;
        slice.copy_from_slice(&self.data[start..(start + 3)]);
        self.index += 3;

        Some(u32::from_le_bytes(bytes))
    }

    pub fn read_f32(&mut self) -> Option<f32> {
        self.read_u32().map(f32::from_bits)
    }

    pub fn read_f64(&mut self) -> Option<f64> {
        self.read_u64().map(f64::from_bits)
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
            let start = self.index;
            bytes.copy_from_slice(&self.data[start..(start + BYTES)]);
            self.index += BYTES;

            Some(<$type>::from_le_bytes(bytes))
        }

        pub fn $iread(&mut self) -> Option<$itype> {
            self.$read().map(|x| x as $itype)
        }
    };
}

impl<'data> Reader<'data> {
    impl_read!(read_u16, u16, read_i16, i16);

    impl_read!(read_u32, u32, read_i32, i32);

    impl_read!(read_u64, u64, read_i64, i64);

    impl_read!(read_u128, u128, read_i128, i128);
}

impl fmt::Debug for Reader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reader")
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "std")]
impl<'data> Read for Reader<'data> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = buf.len().min(self.remaining());

        let start = self.index;
        let slice = &self.data[start..(start + len)];
        buf[0..len].copy_from_slice(slice);

        self.index += len;
        Ok(len)
    }
}

#[must_use]
pub struct Bytes<'data: 'reader, 'reader>(&'reader mut Reader<'data>);

impl Iterator for Bytes<'_, '_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.read_u8()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore() {
        let mut bytes = Reader::new(&[0, 1, 2]);
        bytes.read_u8();
        let restore = bytes.get_restore_point();
        bytes.read_u16();
        assert!(bytes.is_empty());
        bytes.restore(restore);
        assert_eq!(Some(1), bytes.read_u8());
    }

    #[test]
    fn skip_until_any() {
        let mut bytes = Reader::new(&[10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        bytes.read_u8();
        assert!(bytes.skip_until_any(&[10, 9]));
        assert_eq!(Some(9), bytes.read_u8());
    }

    #[test]
    fn skip_until_any_not_found() {
        let mut bytes = Reader::new(&[2, 3, 4]);
        assert!(!bytes.skip_until_any(&[0, 1]));
    }

    #[test]
    fn skip_until_any_no_skip() {
        let mut bytes = Reader::new(&[0]);
        assert!(bytes.skip_until_any(&[0]));
        assert_eq!(Some(0), bytes.read_u8());
        assert!(bytes.is_empty());
    }

    #[test]
    fn read_u16() {
        let mut bytes = Reader::new(&[0x39, 0x05]);
        assert_eq!(Some(0x0539), bytes.read_u16());
    }

    #[test]
    fn read_i16() {
        let mut bytes = Reader::new(&[0xC7, 0xFA]);
        assert_eq!(Some(-0x0539), bytes.read_i16());
    }

    #[test]
    fn read_u24() {
        let mut bytes = Reader::new(&[0x56, 0x34, 0x12]);
        assert_eq!(Some(0x123456), bytes.read_u24());
    }

    #[test]
    fn read_u32() {
        let mut bytes = Reader::new(&[0xEF, 0xCD, 0x34, 0x12]);
        assert_eq!(Some(0x1234_CDEF), bytes.read_u32());
    }

    #[test]
    fn read_i32() {
        let mut bytes = Reader::new(&[0x11, 0x32, 0xCB, 0xED]);
        assert_eq!(Some(-0x1234_CDEF), bytes.read_i32());
    }

    #[test]
    fn read_u64() {
        let mut bytes = Reader::new(&[0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01]);
        assert_eq!(Some(0x0123_4567_89AB_CDEF), bytes.read_u64());
    }

    #[test]
    fn read_i64() {
        let mut bytes = Reader::new(&[1, 0, 0, 0, 0, 0, 0, 0x80]);
        assert_eq!(Some(i64::MIN + 1), bytes.read_i64());
    }

    #[test]
    fn read_u128() {
        let mut bytes = Reader::new(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF]);
        assert_eq!(
            Some(0x0F0E_0D0C_0B0A_0908_0706_0504_0302_0100),
            bytes.read_u128()
        );
    }

    #[test]
    fn read_i128() {
        let mut bytes = Reader::new(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80]);
        assert_eq!(Some(i128::MIN + 1), bytes.read_i128());
    }

    #[test]
    fn bytes_read_line() {
        let mut bytes = Reader::new(&[b'a', 0, b'\n', b'b']);

        assert_eq!(Some(b"a\0".as_ref()), bytes.read_line());
        assert_eq!(Some(b'b'), bytes.read_u8());
    }

    #[test]
    fn bytes_read_n_bytes_exact() {
        let input = [0, 1, 2, 3];

        let mut bytes = Reader::new(&input);

        let read = bytes.read_n_bytes(1);
        assert_eq!(read.len(), 1);
        assert_eq!(read, &input[0..1]);

        let read = bytes.read_n_bytes(0);
        assert_eq!(read.len(), 0);
        assert_eq!(read, &[]);

        let read = bytes.read_n_bytes(3);
        assert_eq!(read.len(), 3);
        assert_eq!(read, &input[1..]);

        assert!(bytes.is_empty());
    }

    #[test]
    fn bytes_read_n_bytes_overshoot() {
        let input = [0];

        let mut bytes = Reader::new(&input);

        let read = bytes.read_n_bytes(2);
        assert_eq!(read.len(), 1);
        assert_eq!(read, input);

        assert!(bytes.is_empty());
    }

    #[test]
    fn bytes_read_line_without_newline() {
        let mut bytes = Reader::new(&[b'a', 0]);

        assert_eq!(Some(b"a\0".as_ref()), bytes.read_line());
        assert_eq!(None, bytes.read_u8());
    }

    #[test]
    fn bytes_read_line_empty() {
        let mut bytes = Reader::new(&[]);
        assert_eq!(None, bytes.read_line());
    }

    #[test]
    #[cfg(feature = "std")]
    fn bytes_read() {
        let input = [0, 1, 2, 3];

        let mut bytes = Reader::new(&input);

        let mut buf = [0; 4];
        let read = bytes.read(&mut buf).unwrap();

        assert_eq!(read, input.len());
        assert_eq!(buf, input);
        assert!(bytes.is_empty());
    }

    #[test]
    #[cfg(feature = "std")]
    fn bytes_read_exact() {
        let input = [0, 1, 2, 3];

        let mut bytes = Reader::new(&input);

        let mut buf = [0; 4];
        bytes.read_exact(&mut buf).unwrap();

        assert_eq!(buf, input);
        assert!(bytes.is_empty());
    }

    #[test]
    #[cfg(feature = "std")]
    fn bytes_read_empty() {
        let mut bytes = Reader::new(&[]);

        let mut buf = [0; 1];
        let read = bytes.read(&mut buf).unwrap();

        assert_eq!([0], buf);
        assert_eq!(read, 0);
    }

    #[test]
    fn bytes_iter() {
        let mut bytes = Reader::new(&[0]);
        let mut iter = bytes.iter();

        assert_eq!(Some(0), iter.next());
        assert_eq!(None, iter.next());
        assert!(bytes.is_empty());
    }
}
