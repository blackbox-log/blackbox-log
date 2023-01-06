use std::alloc::{alloc, dealloc, Layout, LayoutError};
use std::ops::Deref;
use std::slice;

pub(crate) enum OwnedSliceAllocError {
    Layout(LayoutError),
    Alloc,
    ZeroSized,
}

impl From<LayoutError> for OwnedSliceAllocError {
    fn from(error: LayoutError) -> Self {
        Self::Layout(error)
    }
}

pub(crate) struct OwnedSlice {
    ptr: *mut u8,
    len: usize,
}

impl OwnedSlice {
    pub(crate) fn alloc(len: usize) -> Result<*mut u8, OwnedSliceAllocError> {
        let layout = Layout::array::<u8>(len)?;

        if layout.size() == 0 {
            return Err(OwnedSliceAllocError::ZeroSized);
        }

        // SAFETY: above check ensures that the allocation is non-zero-sized
        let ptr = unsafe { alloc(layout) };

        if ptr.is_null() {
            return Err(OwnedSliceAllocError::Alloc);
        }

        Ok(ptr)
    }

    /// Create a new `OwnedSlice` from a pointer and length.
    ///
    /// # Safety
    ///
    /// This must be called with a pointer obtained from [`OwnedSlice::alloc`]
    /// and the same `len`. Any other usage is likely unsound.
    pub(crate) unsafe fn new(ptr: *mut u8, len: usize) -> Self {
        Self { ptr, len }
    }
}

impl Deref for OwnedSlice {
    type Target = [u8];

    // Reason: makes it clearer what lifetime the output slice will have
    #[allow(clippy::needless_lifetimes)]
    fn deref<'a>(&'a self) -> &'a Self::Target {
        // SAFETY: the invariants of `slice::from_raw_parts` are guaranteed to be upheld
        // by callers of `OwnedSlice::new`
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl Drop for OwnedSlice {
    fn drop(&mut self) {
        // SAFETY:
        // - `unwrap_unchecked` is safe because `Layout::array` only returns an error
        //   when its size would exceed `isize::MAX`. Since the alignment of a `u8` is
        //   1, this is only in the case of `self.len > isize::MAX`, which is disallowed
        //   by `OwnedSlice::new`.
        // - the invariants of `dealloc` are guaranteed to be upheld by callers of
        //   `OwnedSlice::new`
        unsafe {
            let layout = Layout::array::<u8>(self.len).unwrap_unchecked();
            dealloc(self.ptr, layout);
        }
    }
}
