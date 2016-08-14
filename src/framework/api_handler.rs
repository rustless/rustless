use typeable::Typeable;
use traitobject;
use std::any::TypeId;
use std::mem;

use backend;
use super::{CallInfo};
use json::{JsonValue};

pub trait ApiHandler: Typeable {
    fn api_call<'a, 'b>(&'a self, &str, &mut JsonValue, &'b mut (backend::Request + 'b), &mut CallInfo<'a>) -> backend::HandleResult<backend::Response>;
}

impl ApiHandler {
    /// Is this `Error` object of type `E`?
    pub fn is<E: ApiHandler>(&self) -> bool { self.get_type() == TypeId::of::<E>() }

    /// If this error is `E`, downcast this error to `E`, by reference.
    pub fn downcast<E: ApiHandler>(&self) -> Option<&E> {
        if self.is::<E>() {
            unsafe { Some(mem::transmute(traitobject::data(self))) }
        } else {
            None
        }
    }

    /// Returns some mutable reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    #[inline]
    pub fn downcast_mut<T: ApiHandler>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe {
                Some(self.downcast_mut_unchecked())
            }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the boxed value, blindly assuming it to be of type `T`.
    /// If you are not *absolutely certain* of `T`, you *must not* call this.
    #[inline]
    pub unsafe fn downcast_mut_unchecked<T: ApiHandler>
                                        (&mut self) -> &mut T {
        mem::transmute(traitobject::data(self))
    }
}

pub type ApiHandlers = Vec<Box<ApiHandler + Send + Sync>>;