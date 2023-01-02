use std::alloc::{alloc, dealloc, Layout, LayoutError};
use std::ops::Deref;
use std::slice;

pub(crate) enum OwnedSliceAllocError {
    Layout(LayoutError),
    Alloc,
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
    pub(crate) unsafe fn alloc(len: usize) -> Result<*mut u8, OwnedSliceAllocError> {
        let layout = get_layout(len)?;
        let ptr = unsafe { alloc(layout) };

        if ptr.is_null() {
            return Err(OwnedSliceAllocError::Alloc);
        }

        Ok(ptr)
    }

    pub(crate) fn new(ptr: *mut u8, len: usize) -> Self {
        Self { ptr, len }
    }
}

impl Deref for OwnedSlice {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl Drop for OwnedSlice {
    fn drop(&mut self) {
        unsafe {
            let layout = get_layout(self.len).unwrap_unchecked();
            dealloc(self.ptr, layout);
        }
    }
}

#[inline]
fn get_layout(len: usize) -> Result<Layout, LayoutError> {
    Layout::array::<u8>(len)
}
