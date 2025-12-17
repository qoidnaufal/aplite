use std::ptr::NonNull;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ArenaPtr<T: ?Sized> {
    raw: NonNull<T>,
}

impl<T: ?Sized> ArenaPtr<T> {
    pub(crate) fn new(raw: *mut T) -> Self {
        Self {
            raw: unsafe { NonNull::new_unchecked(raw) },
        }
    }

    pub fn map<U: ?Sized>(mut self, f: impl FnOnce(&mut T) -> &mut U) -> ArenaPtr<U> {
        ArenaPtr {
            raw: NonNull::from_mut(f(&mut self)),
        }
    }
}

impl<T: ?Sized> std::ops::Deref for ArenaPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.raw.as_ptr()
        }
    }
}

impl<T: ?Sized> std::ops::DerefMut for ArenaPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            self.raw.as_mut()
        }
    }
}

impl<T: ?Sized> AsRef<T> for ArenaPtr<T> {
    fn as_ref(&self) -> &T {
        unsafe {
            self.raw.as_ref()
        }
    }
}

impl<T: ?Sized> AsMut<T> for ArenaPtr<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe {
            self.raw.as_mut()
        }
    }
}

impl<T: ?Sized + std::fmt::Debug> std::fmt::Debug for ArenaPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
}
