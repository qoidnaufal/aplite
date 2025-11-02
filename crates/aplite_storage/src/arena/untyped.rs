use std::num::NonZeroUsize;
use std::alloc;

use super::item::ArenaItem;

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
    pub fn new(size: usize) -> Self {
        let size = NonZeroUsize::try_from(size).expect("Size must be non zero");
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

    pub fn alloc<T>(&mut self, data: T) -> ArenaItem<T> {
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

            assert!(new_offset <= self.start.add(self.size.get()));

            self.offset = new_offset;

            self.drop_calls.push(DropCaller { raw, drop: drop::<T> });

            let ptr = raw.cast();
            std::ptr::write(ptr, data);
            ArenaItem::new(ptr)
        }
    }

    pub fn alloc_mapped<T, U, F>(&mut self, data: T, map: F) -> ArenaItem<U>
    where
        U: ?Sized,
        F: FnOnce(&mut T) -> &mut U,
    {
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

            assert!(new_offset <= self.start.add(self.size.get()));

            self.offset = new_offset;

            self.drop_calls.push(DropCaller { raw, drop: drop::<T> });

            let ptr = raw.cast();
            std::ptr::write(ptr, data);
            ArenaItem::new(map(&mut *ptr))
        }
    }

    // WARN: need to mark the raw pointer in ArenaItem as invalid
    pub fn clear(&mut self) {
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
