use std::num::NonZeroUsize;
use std::alloc;

use super::ptr::Ptr;

pub struct Arena {
    start: *mut u8,
    offset: *mut u8,
    drop_calls: Vec<DropCaller>,
    size: NonZeroUsize,
}

impl Drop for Arena {
    fn drop(&mut self) {
        unsafe {
            self.clear();
            let layout = alloc::Layout::from_size_align_unchecked(self.size.get(), 1);
            alloc::dealloc(self.start, layout);
        }
    }
}

impl Arena {
    pub fn new(size: NonZeroUsize) -> Self {
        unsafe {
            let layout = alloc::Layout::from_size_align_unchecked(size.get(), 1);
            let start = alloc::alloc(layout);
            if start.is_null() {
                alloc::handle_alloc_error(layout);
            }
            Self {
                start,
                offset: start,
                drop_calls: vec![],
                size,
            }
        }
    }

    fn alloc_raw<T>(&mut self, data: T) -> *mut T {
        #[inline]
        unsafe fn drop<T>(raw: *mut u8) {
            unsafe {
                let ptr = raw.cast::<T>();
                ptr.drop_in_place();
            }
        }

        unsafe {
            let aligned_offset = self.offset.align_offset(align_of::<T>());
            let raw = self.offset.add(aligned_offset);
            let new_offset = raw.add(size_of::<T>());

            debug_assert!(new_offset <= self.start.add(self.size.get()));

            self.offset = new_offset;
            self.drop_calls.push(DropCaller { raw, drop: drop::<T> });

            let ptr = raw.cast();
            core::ptr::write(ptr, data);

            ptr
        }
    }

    pub fn alloc<T>(&mut self, data: T) -> Ptr<T> {
        let raw = self.alloc_raw(data);
        Ptr::new(raw)
    }

    pub fn alloc_mapped<T, U, F>(&mut self, data: T, map: F) -> Ptr<U>
    where
        U: ?Sized,
        F: FnOnce(&mut T) -> &mut U,
    {
        let raw = self.alloc_raw(data);
        unsafe {
            Ptr::new(map(&mut *raw))
        }
    }

    pub fn memmove<T>(&mut self, data: *const T) -> Ptr<T> {
        #[inline]
        unsafe fn drop<T>(raw: *mut u8) {
            unsafe {
                let ptr = raw.cast::<T>();
                ptr.drop_in_place();
            }
        }

        unsafe {
            let aligned_offset = self.offset.align_offset(align_of::<T>());
            let raw = self.offset.add(aligned_offset);
            let new_offset = raw.add(size_of::<T>());

            debug_assert!(new_offset <= self.start.add(self.size.get()));

            self.offset = new_offset;
            self.drop_calls.push(DropCaller { raw, drop: drop::<T> });

            let ptr = raw.cast();
            core::ptr::copy_nonoverlapping(data, ptr, 1);
            Ptr::new(ptr)
        }
    }

    // WARN: need to mark the raw pointer in ArenaItem as invalid
    fn clear(&mut self) {
        self.drop_calls.clear();
        self.offset = self.start;
    }

    pub fn size(&self) -> usize {
        self.size.get()
    }

    pub fn used_allocation(&self) -> usize {
        unsafe {
            self.offset.offset_from_unsigned(self.start)
        }
    }

    pub fn remaining(&self) -> usize {
        unsafe {
            let end = self.start.add(self.size.get());
            end.offset_from_unsigned(self.offset)
        }
    }
}

struct DropCaller {
    raw: *mut u8,
    drop: unsafe fn(*mut u8),
}

impl Drop for DropCaller {
    fn drop(&mut self) {
        unsafe {
            (self.drop)(self.raw)
        }
    }
}
