use std::alloc;
use std::rc::Rc;
use std::cell::Cell;

use super::ptr::OwningPtr;

/// Non static Arena-style data storage which will drop all the allocated objects at the same time,
/// not individually from the pointers produced through allocation.
/// The produced pointer has simple safety guard mechanism to check the validity of itself
/// For a simpler, 'static lifetime guaranteed pointer, check [`StaticArena`](super::static_arena::StaticArena).
pub struct Arena {
    start: *mut u8,
    offset: *mut u8,
    size: usize,

    /// This is to ensure each allocated object can be safely dropped
    drop_calls: Vec<DropCaller>,

    /// This is for safety guard to ensure the validity of the pointers produced from allocating
    valid: Rc<Cell<bool>>,
}

impl Drop for Arena {
    fn drop(&mut self) {
        unsafe {
            self.clear();
            let layout = alloc::Layout::from_size_align_unchecked(self.size, 1);
            alloc::dealloc(self.start, layout);
        }
    }
}

impl Arena {
    /// For safety, please ensure the size to allocate is non-zero, and of course, enough :)
    pub fn new(size: usize) -> Self {
        unsafe {
            let layout = alloc::Layout::from_size_align_unchecked(size, 1);
            let start = alloc::alloc(layout);
            if start.is_null() {
                alloc::handle_alloc_error(layout);
            }
            Self {
                start,
                offset: start,
                drop_calls: vec![],
                size,
                valid: Rc::new(Cell::new(true)),
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

            debug_assert!(new_offset <= self.start.add(self.size));

            self.offset = new_offset;
            self.drop_calls.push(DropCaller { raw, drop: drop::<T> });

            let ptr = raw.cast();
            core::ptr::write(ptr, data);

            ptr
        }
    }

    pub fn alloc<T>(&mut self, data: T) -> OwningPtr<T> {
        let raw = self.alloc_raw(data);
        OwningPtr::new(raw, Rc::downgrade(&self.valid))
    }

    // WARN: need to mark the raw pointer in ArenaItem as invalid
    fn clear(&mut self) {
        self.valid.set(false);
        self.drop_calls.clear();
        self.offset = self.start;
        self.valid = Rc::new(Cell::new(true));
    }

    pub const fn size(&self) -> usize {
        self.size
    }

    pub const fn used_allocation(&self) -> usize {
        unsafe {
            self.offset.offset_from_unsigned(self.start)
        }
    }

    pub const fn remaining(&self) -> usize {
        unsafe {
            let end = self.start.add(self.size);
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
