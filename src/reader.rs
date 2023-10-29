use core::fmt;

use crate::utils::as_i8;

/// A wrapper around a byte slice to efficiently read data from a blackbox log.
#[derive(Clone)]
pub(crate) struct Reader<'data> {
    /// Index of the next byte to read
    index: usize,
    data: &'data [u8],
}

/// Opaque type used to rewind a `Reader`.
#[derive(Debug, Clone)]
pub(crate) struct RestorePoint(usize);

impl<'data> Reader<'data> {
    /// Creates a new `Reader` starting at the beginning of `data`.
    ///
    /// # Panics
    ///
    /// This will panic if `data` has a length of `usize::MAX`, since `Reader`
    /// relies on being able to internally store an index `>= data.len()`.
    #[inline]
    #[must_use]
    pub(crate) const fn new(data: &'data [u8]) -> Self {
        if data.len() == usize::MAX {
            panic!("cannot create a Reader containing usize::MAX bytes");
        }

        Self { index: 0, data }
    }

    /// Returns a value that can be passed to [`Reader::restore`] to rewind to
    /// the current index.
    pub(crate) const fn get_restore_point(&self) -> RestorePoint {
        RestorePoint(self.index)
    }

    /// Rewinds to a stored [`RestorePoint`] from [`Reader::get_restore_point`].
    pub(crate) fn restore(&mut self, restore: RestorePoint) {
        self.index = restore.0;
    }

    /// Advances past all bytes not matching any of the needles, returning
    /// `true` if any are found before the end of the buffer.
    pub(crate) fn skip_until_any(&mut self, needles: &[u8]) -> bool {
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

    /// Returns the number of bytes that have not yet been read.
    #[must_use]
    pub(crate) const fn remaining(&self) -> usize {
        self.data.len() - self.index
    }

    /// Returns true if the [`Reader`] has reached the end of the underlying
    /// buffer.
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    /// Returns the next byte without advancing.
    pub(crate) fn peek(&self) -> Option<u8> {
        self.data.get(self.index).copied()
    }

    /// Reads all bytes up to, but not including, the next newline, or the end
    /// of the buffer.
    ///
    /// This is roughly equivalent to `data.iter().take_while(|x| x != b'\n')`,
    /// but is more concise and may be faster.
    pub(crate) fn read_line(&mut self) -> Option<&'data [u8]> {
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

    /// Attempts to read the next `n` bytes, returning the rest of the buffer if
    /// there are fewer than `n` remaining.
    pub(crate) fn read_n_bytes(&mut self, n: usize) -> &'data [u8] {
        let len = n.min(self.remaining());

        let start = self.index;
        let slice = &self.data[start..(start + len)];

        self.index += len;
        slice
    }

    /// Reads a single byte as a `u8`.
    pub(crate) fn read_u8(&mut self) -> Option<u8> {
        let byte = self.peek();
        if byte.is_some() {
            self.index += 1;
        }
        byte
    }

    /// Reads a single byte as an `i8`.
    pub(crate) fn read_i8(&mut self) -> Option<i8> {
        self.read_u8().map(as_i8)
    }

    /// Reads 3 bytes as the lower bytes of a `u32`.
    pub(crate) fn read_u24(&mut self) -> Option<u32> {
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

    pub(crate) fn read_f32(&mut self) -> Option<f32> {
        self.read_u32().map(f32::from_bits)
    }
}

macro_rules! impl_read {
    ($read:ident, $type:ty, $iread:ident, $itype:ty) => {
        #[allow(dead_code)]
        pub(crate) fn $read(&mut self) -> Option<$type> {
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

        #[allow(dead_code)]
        pub(crate) fn $iread(&mut self) -> Option<$itype> {
            self.$read().map(|x| x as $itype)
        }
    };
}

impl Reader<'_> {
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
}
