use std::cell::UnsafeCell;
use std::alloc;

use super::ptr::ArenaPtr;

thread_local! {
    static ARENA: UnsafeCell<Inner> = UnsafeCell::new(Inner::new());
}

pub struct Inner {
    start: *mut u8,
    offset: *mut u8,
    size: usize,
}

impl Inner {
    const fn new() -> Self {
        Self {
            start: std::ptr::null_mut(),
            offset: std::ptr::null_mut(),
            size: 0,
        }
    }
}

pub struct StaticArena;

// impl Drop for Arena {
//     fn drop(&mut self) {
//         unsafe {
//             self.clear();
//             let layout = alloc::Layout::from_size_align_unchecked(self.size.get(), 1);
//             alloc::dealloc(self.start, layout);
//         }
//     }
// }

impl StaticArena {
    /// For safety, please ensure the size to allocate is non-zero and, of course, enough :)
    pub fn init(size: usize) {
        ARENA.with(|arena| unsafe {
            let layout = alloc::Layout::from_size_align_unchecked(size, 1);
            let start = alloc::alloc(layout);
            if start.is_null() {
                alloc::handle_alloc_error(layout);
            }

            *arena.get() = Inner {
                start,
                offset: start,
                size,
            };
        });
    }

    #[inline(always)]
    fn alloc_raw<T>(data: T) -> *mut T {
        // #[inline]
        // unsafe fn drop<T>(raw: *mut u8) {
        //     unsafe {
        //         let ptr = raw.cast::<T>();
        //         ptr.drop_in_place();
        //     }
        // }

        ARENA.with(|cell|  unsafe {
            let arena = &mut *cell.get();
            let aligned_offset = arena.offset.align_offset(align_of::<T>());
            let raw = arena.offset.add(aligned_offset);
            let new_offset = raw.add(size_of::<T>());

            debug_assert!(new_offset <= arena.start.add(arena.size));

            arena.offset = new_offset;
            // arena.drop_calls.push(DropCaller { raw, drop: drop::<T> });

            let ptr = raw.cast();
            core::ptr::write(ptr, data);

            ptr
        })
    }

    pub fn alloc<T>(data: T) -> ArenaPtr<T> {
        let raw = Self::alloc_raw(data);
        ArenaPtr::new(raw)
    }

    // WARN: need to mark the raw pointer in ArenaItem as invalid
    // fn clear(&mut self) {
    //     self.drop_calls.clear();
    //     self.offset = self.start;
    // }

    pub fn size() -> usize {
        ARENA.with(|cell| unsafe {
            let arena = &*cell.get();
            arena.size
        })
    }

    pub fn used_allocation() -> usize {
        ARENA.with(|cell| unsafe {
            let arena = &*cell.get();
            arena.offset.offset_from_unsigned(arena.start)
        })
    }

    pub fn remaining(&self) -> usize {
        ARENA.with(|cell| unsafe {
            let arena = &*cell.get();
            let end = arena.start.add(arena.size);
            end.offset_from_unsigned(arena.offset)
        })
    }
}

// struct DropCaller {
//     raw: *mut u8,
//     drop: unsafe fn(*mut u8),
// }

// impl Drop for DropCaller {
//     fn drop(&mut self) {
//         unsafe {
//             (self.drop)(self.raw)
//         }
//     }
// }
