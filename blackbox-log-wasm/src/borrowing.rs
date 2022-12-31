use std::mem;
use std::ops::Deref;
use std::rc::Rc;

// SAFETY: since data is second, it will be dropped second, meaning borrower's
// reference will not dangle
pub(crate) struct Borrowing<T> {
    borrower: T,
    #[allow(unused)]
    data: Rc<Box<[u8]>>,
}

impl<T> Borrowing<T> {
    pub(crate) fn new(data: Box<[u8]>, new: impl FnOnce(&'static [u8]) -> T) -> Self {
        let data = Rc::new(data);
        // SAFETY: ???
        let data_ref: &'static Box<[u8]> = unsafe { mem::transmute(data.deref()) };
        Self {
            borrower: new(data_ref),
            data,
        }
    }

    pub(crate) fn new_borrow<U>(&self, new: impl FnOnce(&T) -> U) -> Borrowing<U> {
        Borrowing {
            borrower: new(&self.borrower),
            data: self.data.clone(),
        }
    }

    pub(crate) fn map<U>(self, map: impl FnOnce(T) -> U) -> Borrowing<U> {
        Borrowing {
            borrower: map(self.borrower),
            data: self.data,
        }
    }
}

impl<T> Deref for Borrowing<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.borrower
    }
}
