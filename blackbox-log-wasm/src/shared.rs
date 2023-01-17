use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct Shared<T>(Pin<Rc<T>>);

impl<T> Shared<T> {
    pub(crate) fn new(borrowee: T) -> Self {
        Self(Rc::pin(borrowee))
    }

    /// Gets a `'static` reference to the inner data.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that the resulting reference cannot outlive
    /// the `Shared` it came from.
    pub(crate) unsafe fn deref_static<'a>(&'a self) -> &'static T {
        std::mem::transmute::<&'a T, &'static T>(self.deref())
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(Pin::clone(&self.0))
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn invalid_deref_static() {
        let x = Shared::new("test".to_owned());

        // SAFETY: x is dropped only after x_ref's last use. Switching the last 2 lines
        // is UB.
        let x_ref = unsafe { x.deref_static() };

        assert_eq!("test", x_ref);
        drop(x);
    }

    #[test]
    fn clone() {
        let x = Shared::new(());
        let _x2 = Shared::clone(&x);
        assert_eq!(2, Rc::strong_count(&Pin::into_inner(x.0)));
    }
}
