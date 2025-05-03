#![expect(unsafe_code)]

use alloc::boxed::Box;
use core::marker::PhantomData;
use core::ops;
use core::ptr::NonNull;

use crate::utils::{as_i32, as_isize};

#[derive(Debug)]
pub(super) struct FrameHistory {
    data: Box<[u32]>,
    current: u8,
    past: u8,
}

impl FrameHistory {
    pub(super) fn new(len: usize) -> Self {
        Self {
            data: alloc::vec![0; len * 3].into_boxed_slice(),
            current: 0,
            past: 0,
        }
    }

    fn len(&self) -> usize {
        self.data.len() / 3
    }

    pub(super) fn finish(&mut self) -> &[u32] {
        let raw = self.map_range(..);
        self.current = (self.current + 1) % 3;
        self.past = self.past.saturating_add(1);
        &self.data[raw]
    }

    pub(super) fn iter(&mut self) -> impl Iterator<Item = (&mut u32, History<u32>)> {
        let len = self.len();
        let current = usize::from(self.current) * len;
        let last = usize::from((self.current + 1) % 3) * len;
        let last_last = usize::from((self.current + 2) % 3) * len;

        let ptr = NonNull::new(self.data.as_mut_ptr()).unwrap();
        unsafe {
            Iter {
                current: ptr.add(current),
                last: ptr.add(last),
                last_last: ptr.add(last_last),
                i: 0,
                len,
                past: self.past,
                _inner: PhantomData,
            }
        }
    }

    fn map_range<R: ops::RangeBounds<usize>>(&self, range: R) -> ops::Range<usize> {
        let len = self.len();
        let offset = usize::from(self.current) * len;

        let start = match range.start_bound() {
            ops::Bound::Included(x) => offset + x,
            ops::Bound::Excluded(x) => offset + x + 1,
            ops::Bound::Unbounded => offset,
        };
        let end = match range.end_bound() {
            ops::Bound::Included(x) => offset + x + 1,
            ops::Bound::Excluded(x) => offset + x,
            ops::Bound::Unbounded => offset + len,
        };

        start..end
    }
}

impl<I: ops::RangeBounds<usize>> ops::Index<I> for FrameHistory {
    type Output = [u32];

    fn index(&self, index: I) -> &Self::Output {
        &self.data[self.map_range(index)]
    }
}

impl<I: ops::RangeBounds<usize>> ops::IndexMut<I> for FrameHistory {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        let index = self.map_range(index);
        &mut self.data[index]
    }
}

struct Iter<'a> {
    current: NonNull<u32>,
    last: NonNull<u32>,
    last_last: NonNull<u32>,
    i: usize,
    len: usize,
    past: u8,
    _inner: PhantomData<&'a mut FrameHistory>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a mut u32, History<u32>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.len {
            return None;
        }

        let i = as_isize(self.i);
        self.i += 1;

        Some(unsafe {
            let current = self.current.offset(i).as_mut();
            let last = self.last.offset(i).read();
            let last_last = self.last_last.offset(i).read();
            (current, History::new(self.past, last, last_last))
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum History<T> {
    None,
    One(T),
    Two(T, T),
}

impl<T> History<T> {
    fn new(past: u8, last: T, last_last: T) -> Self {
        match past {
            0 => Self::None,
            1 => Self::One(last),
            _ => Self::Two(last, last_last),
        }
    }

    pub(crate) fn last_or(&self, default: T) -> T
    where
        T: Copy,
    {
        match self {
            Self::None => default,
            Self::One(last) | Self::Two(last, _) => *last,
        }
    }
}

impl History<u32> {
    pub(crate) fn as_i32(self) -> History<i32> {
        match self {
            History::None => History::None,
            History::One(last) => History::One(as_i32(last)),
            History::Two(last, last_last) => History::Two(as_i32(last), as_i32(last_last)),
        }
    }
}
