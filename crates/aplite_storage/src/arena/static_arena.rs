use std::sync::{OnceLock, RwLock};
use std::alloc;

use super::ptr::ArenaPtr;

static ARENA: OnceLock<RwLock<MemoryBlock>> = OnceLock::new();

pub struct MemoryBlock {
    start: *mut u8,
    offset: *mut u8,
    size: usize,
}

unsafe impl Send for MemoryBlock {}
unsafe impl Sync for MemoryBlock {}

impl MemoryBlock {
    unsafe fn init(size: usize) -> Self {
        unsafe {
            let layout = alloc::Layout::from_size_align_unchecked(size, 1);
            let start = alloc::alloc(layout);
            if start.is_null() {
                alloc::handle_alloc_error(layout);
            }

            Self {
                start,
                offset: start,
                size,
            }
        }
    }

    unsafe fn deinit(&mut self) {
        unsafe {
            let layout = alloc::Layout::from_size_align_unchecked(self.size, 1);
            std::alloc::dealloc(self.start, layout);
        }
    }
}

/// A `'static` Arena-style which will be allocated at the begining, and cleaned-up all at once.
/// This is for any kind of object you don't intend to keep throughout the whole program.
/// Therefore there's no `.clear()` method for this, and the returned pointer is not dropable
/// 
/// # Example
/// 
/// ```ignore
/// use StaticArena;
/// 
/// struct Obj {
///     name: String,
///     age: u8,
/// }
/// 
/// fn main() {
///     StaticArena::init(1024);
/// 
///     let obj = StaticArena::alloc(Obj { name: "obj", age: 69 });
///
///     /* do something else */
/// 
/// } /* allocations will be deallocated all at once here */
/// ```
pub struct StaticArena;

impl StaticArena {
    /// For safety, please ensure the size to allocate is non-zero, and of course, enough :)
    pub fn init(size: usize) {
        unsafe {
            if let Err(lock) = ARENA.set(RwLock::new(MemoryBlock::init(size))) {
                lock.write().unwrap().deinit();
            }
        }
    }

    #[inline(always)]
    fn alloc_raw<T>(data: T) -> *mut T {
        unsafe {
            let arena = &mut *ARENA.get().unwrap().write().unwrap();
            let aligned_offset = arena.offset.align_offset(align_of::<T>());
            let raw = arena.offset.add(aligned_offset);
            let new_offset = raw.add(size_of::<T>());

            debug_assert!(new_offset <= arena.start.add(arena.size));

            arena.offset = new_offset;

            let ptr = raw.cast();
            core::ptr::write(ptr, data);

            ptr
        }
    }

    pub fn alloc<T>(data: T) -> ArenaPtr<T> {
        let raw = Self::alloc_raw(data);
        ArenaPtr::new(raw)
    }

    pub fn size() -> usize {
        ARENA.get().unwrap().read().unwrap().size
    }

    pub fn used_allocation() -> usize {
        unsafe {
            let arena = &*ARENA.get().unwrap().read().unwrap();
            arena.offset.offset_from_unsigned(arena.start)
        }
    }

    pub fn remaining(&self) -> usize {
        unsafe {
            let arena = &*ARENA.get().unwrap().read().unwrap();
            let end = arena.start.add(arena.size);
            end.offset_from_unsigned(arena.offset)
        }
    }
}
