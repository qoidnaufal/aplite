pub struct ArenaItem<T: ?Sized>(pub(crate) *mut T);

impl<T: ?Sized> Clone for ArenaItem<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T: ?Sized> Copy for ArenaItem<T> {}

impl<T: ?Sized> ArenaItem<T> {
    pub fn map<U: ?Sized>(mut self, f: impl FnOnce(&mut T) -> &mut U) -> ArenaItem<U> {
        ArenaItem(f(&mut self))
    }
}

impl<T: ?Sized> std::ops::Deref for ArenaItem<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.0
        }
    }
}

impl<T: ?Sized> std::ops::DerefMut for ArenaItem<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.0
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
