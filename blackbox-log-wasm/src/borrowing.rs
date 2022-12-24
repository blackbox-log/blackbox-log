use std::mem;
use std::ops::Deref;
use std::rc::Rc;

// SAFETY: since data is second, it will be dropped second, meaning borrower's
// reference will not dangle
pub(crate) struct Borrowing<T, D = Box<[u8]>> {
    borrower: T,
    #[allow(unused)]
    data: Rc<D>,
}

impl<T, D> Borrowing<T, D> {
    pub(crate) fn new(data: D, new: impl FnOnce(&'static D) -> T) -> Self
    where
        D: 'static,
    {
        let data = Rc::new(data);
        // SAFETY: ???
        let data_ref: &'static _ = unsafe { mem::transmute(data.deref()) };
        Self {
            borrower: new(data_ref),
            data,
        }
    }

    pub(crate) fn new_borrow<U>(&self, new: impl FnOnce(&T) -> U) -> Borrowing<U, D> {
        Borrowing {
            borrower: new(&self.borrower),
            data: self.data.clone(),
        }
    }

    pub(crate) fn map<U>(self, map: impl FnOnce(T) -> U) -> Borrowing<U, D> {
        Borrowing {
            borrower: map(self.borrower),
            data: self.data,
        }
    }
}

impl<T, D> Borrowing<Option<T>, D> {
    pub(crate) fn transpose(self) -> Option<Borrowing<T, D>> {
        Some(Borrowing {
            borrower: self.borrower?,
            data: self.data,
        })
    }
}

impl<T, D> Deref for Borrowing<T, D> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.borrower
    }
}
