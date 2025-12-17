use std::ptr::NonNull;
use std::cell::Cell;
use std::rc::Weak;

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

    fn get(&self) -> &T {
        self.as_ref()
    }
}

pub struct OwningPtr<T: ?Sized> {
    raw: NonNull<T>,
    valid: Weak<Cell<bool>>,
}

impl<T: ?Sized> OwningPtr<T> {
    pub(crate) fn new(raw: *mut T, valid: Weak<Cell<bool>>) -> Self {
        Self {
            raw: unsafe { NonNull::new_unchecked(raw) },
            valid,
        }
    }

    /// Return None if the pointer is no longer valid
    pub fn map<U: ?Sized>(mut self, f: impl FnOnce(&mut T) -> &mut U) -> Option<OwningPtr<U>> {
        Some(OwningPtr {
            raw: NonNull::from_mut(f(self.get_mut()?)),
            valid: Weak::clone(&self.valid),
        })
    }

    /// Return None if the pointer is no longer valid
    pub fn get(&self) -> Option<&T> {
        self.valid.upgrade()
            .and_then(|valid| valid.get().then_some(unsafe {
                self.raw.as_ref()
            }))
    }

    /// Return None if the pointer is no longer valid
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.valid.upgrade()
            .and_then(|valid| valid.get().then_some(unsafe {
                self.raw.as_mut()
            }))
    }
}

macro_rules! impl_debug_ptr {
    ($ptr:ident) => {
        impl<T: ?Sized + std::fmt::Debug> std::fmt::Debug for $ptr<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self.get())
            }
        }
    };
}

impl_debug_ptr!(ArenaPtr);
impl_debug_ptr!(OwningPtr);

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
