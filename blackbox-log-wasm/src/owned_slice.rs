use std::alloc::{alloc, alloc_zeroed, dealloc, Layout, LayoutError};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::{mem, slice};

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

// SAFETY: requires multi-value returns
unsafe impl<T> crate::WasmSafe for OwnedSlice<T> {}

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
    /// This must be called with a pointer obtained from [`OwnedSlice::alloc`]
    /// and the same `len`. Any other usage is likely unsound. All `len`
    /// elements *must* be initialized before this call.
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
        // by callers of `OwnedSlice::new`
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for OwnedSlice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: the invariants of `slice::from_raw_parts_mut` are guaranteed to be
        // upheld by callers of `OwnedSlice::new`
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> Drop for OwnedSlice<T> {
    fn drop(&mut self) {
        let layout = Self::layout(self.len).unwrap();
        let ptr = self.ptr.as_ptr() as *mut u8;

        // SAFETY: the invariants of `dealloc` are guaranteed to be upheld by callers of
        // `OwnedSlice::new`
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
