use std::num::NonZeroUsize;
use std::alloc;

use super::item::ArenaItem;

pub struct Arena {
    start: *mut u8,
    offset: *mut u8,
    elements: Vec<Element>,
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
                elements: Vec::new(),
                size,
            }
        }
    }

    pub fn insert<T>(&mut self, data: T) -> ArenaItem<T> {
        #[inline(always)]
        unsafe fn drop<T>(raw: *mut u8) {
            unsafe {
                std::ptr::drop_in_place(raw.cast::<T>());
            }
        }

        unsafe {
            let layout = alloc::Layout::new::<T>();
            let offset = self.offset.align_offset(layout.align());
            let raw = self.offset.add(offset);

            assert!(raw <= self.start.add(self.size.get()));
            assert!(!raw.is_null());

            self.offset = raw.add(layout.size());
            self.elements.push(Element {
                raw,
                drop: drop::<T>
            });

            let ptr = raw.cast();
            std::ptr::write(ptr, data);
            ArenaItem(ptr)
        }
    }

    // WARN: need to mark the raw pointer in ArenaItem as invalid
    pub fn clear(&mut self) {
        self.elements.clear();
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

struct Element {
    raw: *mut u8,
    drop: unsafe fn(*mut u8),
}

impl Drop for Element {
    fn drop(&mut self) {
        unsafe {
            (self.drop)(self.raw)
        }
    }
}
