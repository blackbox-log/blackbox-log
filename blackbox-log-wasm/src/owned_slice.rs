use std::alloc::{alloc, alloc_zeroed, dealloc, Layout, LayoutError};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::{mem, slice};

use crate::{Structural, WasmByValue};

#[derive(Debug)]
pub(crate) enum AllocError {
    Layout(LayoutError),
    Alloc,
    ZeroSized,
}

impl From<LayoutError> for AllocError {
    fn from(error: LayoutError) -> Self {
        Self::Layout(error)
    }
}

#[repr(C)]
pub(crate) struct OwnedSlice<T> {
    len: usize,
    ptr: NonNull<T>,
}

// SAFETY: just two usizes when passed by value
unsafe impl<T: Structural> WasmByValue for OwnedSlice<T> {}

impl<T> OwnedSlice<T> {
    pub(crate) fn new_zeroed(len: usize) -> Self {
        let layout = Self::layout(len).unwrap();

        let ptr = if len == 0 || layout.size() == 0 {
            NonNull::dangling()
        } else {
            // SAFETY: above branch ensures that the allocation is non-zero-sized
            let ptr = unsafe { alloc_zeroed(layout) } as *mut T;
            NonNull::new(ptr).unwrap()
        };

        Self { len, ptr }
    }

    /// Allocate uninitialized backing storage for an `OwnedSlice`.
    pub(crate) fn alloc(len: usize) -> Result<*mut T, AllocError> {
        let layout = Self::layout(len)?;

        if len == 0 || layout.size() == 0 {
            return Err(AllocError::ZeroSized);
        }

        // SAFETY: above check ensures that the allocation is non-zero-sized
        let ptr = unsafe { alloc(layout) } as *mut T;

        if ptr.is_null() {
            return Err(AllocError::Alloc);
        }

        Ok(ptr)
    }

    /// Create a new `OwnedSlice` from a length and pointer.
    ///
    /// # Safety
    ///
    /// - `ptr` must be properly aligned and point to `len` properly initialized
    ///   values of type `T`
    /// - if `len` is zero, `ptr` must be an aligned dangling pointer and it
    ///   will not be deallocated
    /// - `ptr` must point to the beginning of a single contiguous object
    ///   allocated by the global allocator
    /// - there must be no access to the backing memory outside of values
    ///   returned by `OwnedSlice`
    /// - `len * mem::size_of::<T>()` must be no larger than `isize::MAX`
    ///
    /// See the safety documentation of [`std::slice::from_raw_parts_mut`] &
    /// [`std::alloc::dealloc`][`dealloc`].
    pub(crate) unsafe fn from_raw_parts(len: usize, ptr: NonNull<T>) -> Self {
        debug_assert_eq!(0, ptr.as_ptr().align_offset(mem::align_of::<T>()));

        Self { len, ptr }
    }

    #[inline(always)]
    fn layout(len: usize) -> Result<Layout, LayoutError> {
        Layout::array::<T>(len)
    }
}

impl<T> Default for OwnedSlice<T> {
    fn default() -> Self {
        Self::new_zeroed(0)
    }
}

impl<T> Deref for OwnedSlice<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // SAFETY: the invariants of `slice::from_raw_parts` are guaranteed to be upheld
        // by callers of `OwnedSlice::from_raw_parts`
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for OwnedSlice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: the invariants of `slice::from_raw_parts_mut` are guaranteed to be
        // upheld by callers of `OwnedSlice::from_raw_parts`
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> Drop for OwnedSlice<T> {
    fn drop(&mut self) {
        let layout = Self::layout(self.len).unwrap();
        let ptr = self.ptr.as_ptr() as *mut u8;

        // SAFETY: the invariants of `dealloc` are guaranteed to be upheld by callers of
        // `OwnedSlice::from_raw_parts`
        unsafe { dealloc(ptr, layout) }
    }
}

impl<T> From<Vec<T>> for OwnedSlice<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::from(vec.into_boxed_slice())
    }
}

impl<T> From<Box<[T]>> for OwnedSlice<T> {
    fn from(slice: Box<[T]>) -> Self {
        let len = slice.len();
        let ptr = Box::into_raw(slice).cast();
        let ptr = NonNull::new(ptr).unwrap();

        // SAFETY:
        // - proper alignment, allocation, etc is guaranteed because `ptr` comes from an
        //   existing slice
        // - taking ownership of the Box prevents invalid usage of pointer
        unsafe { Self::from_raw_parts(len, ptr) }
    }
}

wasm_export!(free slice8_free: OwnedSlice<u8>);

#[cfg(test)]
mod test {
    use std::mem;

    use super::*;

    #[test]
    fn option_niche() {
        assert_eq!(
            mem::size_of::<OwnedSlice<()>>(),
            mem::size_of::<Option<OwnedSlice<()>>>()
        );
        assert_eq!(
            mem::align_of::<OwnedSlice<()>>(),
            mem::align_of::<Option<OwnedSlice<()>>>()
        );
    }
}
