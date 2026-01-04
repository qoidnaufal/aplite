use core::mem::{self, ManuallyDrop};

// It's important to use union over enum to be more size-efficient
pub(crate) union Content<T> {
    pub(crate) data: ManuallyDrop<T>,
    pub(crate) next_id: u32,
}

pub(crate) struct Slot<T> {
    pub(crate) content: Content<T>,

    /// Similar with the original [`slotmap`](https://crates.io/crates/slotmap) crates, in which only u32::MAX / 2 versions are available, and it should be plenty enough
    pub(crate) version: u32,
}

impl<T> Slot<T> {
    #[inline(always)]
    pub(crate) fn new(data: T) -> Self {
        Self {
            content: Content { data: ManuallyDrop::new(data) },
            version: 0,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.version % 2 != 0
    }

    pub(crate) fn validate_occupied(&self, version: u32) -> bool {
        self.version % 2 == 0 && self.version == version
    }

    pub(crate) fn get(&self) -> Option<&T> {
        unsafe {
            if !self.is_empty() {
                Some(&self.content.data)
            } else {
                None
            }
        }
    }

    pub(crate) fn get_validated(&self, version: u32) -> Option<&T> {
        unsafe {
            self.validate_occupied(version).then_some(&self.content.data)
        }
    }

    pub(crate) fn get_mut(&mut self) -> Option<&mut T> {
        unsafe {
            if !self.is_empty() {
                Some(&mut self.content.data)
            } else {
                None
            }
        }
    }

    pub(crate) fn get_validated_mut(&mut self, version: u32) -> Option<&mut T> {
        unsafe {
            self.validate_occupied(version).then_some(&mut self.content.data)
        }
    }

    pub(crate) fn set_vacant(&mut self, next_id: u32) -> T {
        self.version += 1;
        debug_assert_ne!(self.version % 2, 0);

        unsafe {
            let content = mem::replace(&mut self.content, Content { next_id });
            ManuallyDrop::into_inner(content.data)
        }
    }

    pub(crate) fn occupy(&mut self, data: T) {
        self.content = Content { data: ManuallyDrop::new(data) };
        self.version += 1;
        debug_assert_eq!(self.version % 2, 0);
    }
}

impl<T> Drop for Slot<T> {
    fn drop(&mut self) {
        if !self.is_empty() {
            unsafe {
                ManuallyDrop::drop(&mut self.content.data);
            }
        }
    }
}

impl<T: Clone> Clone for Slot<T> {
    fn clone(&self) -> Self {
        Self {
            content: unsafe {
                if self.is_empty() {
                    Content { next_id: self.content.next_id }
                } else {
                    Content { data: self.content.data.clone() }
                }
            },
            version: self.version,
        }
    }
}
