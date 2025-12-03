use std::ptr::NonNull;

#[repr(transparent)]
pub struct Ptr<T: ?Sized> {
    raw: NonNull<T>,
}

impl<T: ?Sized> Ptr<T> {
    pub(crate) fn new(raw: *mut T) -> Self {
        Self {
            raw: unsafe { NonNull::new_unchecked(raw) },
        }
    }

    pub fn map<U: ?Sized>(&mut self, f: impl FnOnce(&mut T) -> &mut U) -> Ptr<U> {
        Ptr {
            raw: NonNull::from_mut(f(self)),
        }
    }
}

impl<T: ?Sized> std::ops::Deref for Ptr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.raw.as_ptr()
        }
    }
}

impl<T: ?Sized> std::ops::DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            self.raw.as_mut()
        }
    }
}

impl<T: ?Sized> AsRef<T> for Ptr<T> {
    fn as_ref(&self) -> &T {
        std::ops::Deref::deref(self)
    }
}

impl<T: ?Sized> AsMut<T> for Ptr<T> {
    fn as_mut(&mut self) -> &mut T {
        std::ops::DerefMut::deref_mut(self)
    }
}

impl<T: ?Sized + std::fmt::Debug> std::fmt::Debug for Ptr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
}

// impl<T: ?Sized> Drop for Ptr<T> {
//     fn drop(&mut self) {
//         unsafe {
//             std::ptr::drop_in_place(self.raw.as_ptr());
//         }
//     }
// }
