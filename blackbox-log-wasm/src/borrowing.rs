use std::borrow::Borrow;
use std::mem;
use std::ops::Deref;
use std::rc::Rc;

use crate::OwnedSlice;

// SAFETY: since data is second, it will be dropped second, meaning borrower's
// reference will not dangle
pub(crate) struct Borrowing<T, D = OwnedSlice> {
    borrower: T,
    #[allow(unused)]
    data: Rc<D>,
}

impl<T, D> Borrowing<T, D> {
    pub(crate) fn new<E>(data: D, new: impl FnOnce(&'static E) -> T) -> Self
    where
        E: 'static,
        D: Borrow<E>,
    {
        let data = Rc::new(data);
        Self::new_with_rc(data, new)
    }

    pub(crate) fn new_with_rc<E>(data: Rc<D>, new: impl FnOnce(&'static E) -> T) -> Self
    where
        E: 'static,
        D: Borrow<E>,
    {
        let data_ref: &E = data.deref().borrow();
        let data_ref: &'static E = unsafe { mem::transmute(data_ref) };
        Self {
            borrower: new(data_ref),
            data,
        }
    }
}

impl<T, D> Borrowing<T, D> {
    pub(crate) fn new_borrow<U>(&self, new: impl FnOnce(&T) -> U) -> Borrowing<U, D> {
        Borrowing {
            borrower: new(&self.borrower),
            data: Rc::clone(&self.data),
        }
    }

    pub(crate) fn map<U>(self, map: impl FnOnce(T) -> U) -> Borrowing<U, D> {
        Borrowing {
            borrower: map(self.borrower),
            data: self.data,
        }
    }
}

impl<T, D> Deref for Borrowing<T, D> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.borrower
    }
}
