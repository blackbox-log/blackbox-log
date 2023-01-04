use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct Shared<T>(Pin<Rc<T>>);

impl<T> Shared<T> {
    pub(crate) fn new(borrowee: T) -> Self {
        Self(Rc::pin(borrowee))
    }

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
