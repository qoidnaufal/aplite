use std::ptr::NonNull;

pub struct ArenaItem<T: ?Sized> {
    raw: NonNull<T>,
    marker: std::marker::PhantomData<T>,
}

impl<T: ?Sized> ArenaItem<T> {
    pub(crate) fn new(raw: *mut T) -> Self {
        Self {
            raw: unsafe { NonNull::new_unchecked(raw) },
            marker: std::marker::PhantomData
        }
    }

    pub fn map<U: ?Sized>(mut self, f: impl FnOnce(&mut T) -> &mut U) -> ArenaItem<U> {
        ArenaItem {
            raw: NonNull::from_mut(f(&mut self)),
            marker: std::marker::PhantomData,
        }
    }
}

impl<T: ?Sized> std::ops::Deref for ArenaItem<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.raw.as_ptr()
        }
    }
}

impl<T: ?Sized> std::ops::DerefMut for ArenaItem<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            self.raw.as_mut()
        }
    }
}

impl<T: ?Sized> AsRef<T> for ArenaItem<T> {
    fn as_ref(&self) -> &T {
        std::ops::Deref::deref(self)
    }
}

impl<T: ?Sized> AsMut<T> for ArenaItem<T> {
    fn as_mut(&mut self) -> &mut T {
        std::ops::DerefMut::deref_mut(self)
    }
}

impl<T: ?Sized + std::fmt::Debug> std::fmt::Debug for ArenaItem<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
