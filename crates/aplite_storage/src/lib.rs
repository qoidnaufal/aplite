mod buffer;
mod map;
mod tree;

pub use buffer::*;

pub use tree::{
    sparse_tree::{SparseTree, TreeError},
    node::{Node, NodeRef, SubTree},
};

pub use map::{
    id::SlotId,
    slot_map::{SlotMap, Error}
};

pub const fn needs_drop<T>() -> Option<unsafe fn(*mut u8, usize)> {
    #[inline]
    unsafe fn drop<T>(raw: *mut u8, len: usize) {
        unsafe {
            std::ptr::slice_from_raw_parts_mut(raw.cast::<T>(), len).drop_in_place();
        }
    }

    if std::mem::needs_drop::<T>() {
        Some(drop::<T>)
    } else {
        None
    }
}
