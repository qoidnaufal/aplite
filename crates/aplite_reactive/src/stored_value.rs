// use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, LockResult};
use std::cell::UnsafeCell;

pub(crate) struct Value<T> {
    // inner: Arc<RwLock<T>>,
    inner: UnsafeCell<T>,
}

unsafe impl<T> Send for Value<T> {}
unsafe impl<T> Sync for Value<T> {}

impl<T> Value<T> {
    #[inline(always)]
    pub(crate) fn new(value: T) -> Self {
        Self {
            // inner: Arc::new(RwLock::new(value)),
            inner: UnsafeCell::new(value),
        }
    }

    pub(crate) fn read(&self) -> &T {
        unsafe {
            &*self.inner.get()
        }
    }

    pub(crate) fn write(&self) -> &mut T {
        unsafe {
            &mut *self.inner.get()
        }
    }

    // #[inline(always)]
    // pub(crate) fn read<'a>(&'a self) -> LockResult<RwLockReadGuard<'a, T>> {
    //     self.inner.read()
    // }

    // #[inline(always)]
    // pub(crate) fn write<'a>(&'a self) -> LockResult<RwLockWriteGuard<'a, T>> {
    //     self.inner.write()
    // }
}

impl<T: std::fmt::Debug + 'static> std::fmt::Debug for Value<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .field("value", &self.read())
            .finish()
    }
}
