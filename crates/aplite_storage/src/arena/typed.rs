use std::alloc;
use std::marker::PhantomData;
use std::ptr::slice_from_raw_parts_mut;

use super::ptr::Ptr;

pub struct TypedArena<T> {
    block: *mut u8,
    capacity: usize,
    len: usize,
    marker: PhantomData<T>,
}

impl<T> Drop for TypedArena<T> {
    fn drop(&mut self) {
        unsafe {
            self.clear();
            let size = size_of::<T>() * self.capacity;
            let align = align_of::<T>();
            let layout = alloc::Layout::from_size_align_unchecked(size, align);
            alloc::dealloc(self.block, layout);
        }
    }
}

impl<T> TypedArena<T> {
    pub fn new(capacity: usize) -> Self {
        unsafe {
            let size = size_of::<T>() * capacity;
            let align = align_of::<T>();
            let layout = alloc::Layout::from_size_align_unchecked(size, align);
            let block = alloc::alloc(layout);
            if block.is_null() {
                alloc::handle_alloc_error(layout);
            }
            Self {
                block,
                capacity,
                len: 0,
                marker: PhantomData,
            }
        }
    }

    pub fn insert(&mut self, data: T) -> Ptr<T> {
        assert!(self.len < self.capacity, "max capacity reached");
        unsafe {
            let raw = self.block.add(self.len * size_of::<T>()).cast();
            self.len += 1;
            std::ptr::write(raw, data);

            Ptr::new(raw)
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len { return None }
        unsafe {
            let ptr = self.block.add(index * size_of::<T>()).cast();
            Some(&*ptr)
        }
    }

    pub fn get_ptr(&self, index: usize) -> Ptr<T> {
        assert!(index < self.len);
        unsafe {
            let raw = self.block.add(index * size_of::<T>()).cast();
            Ptr::new(raw)
        }
    }

    pub fn last(&self) -> Option<&T> {
        if self.len == 0 { return None }
        unsafe {
            let ptr = self.block.add((self.len - 1) * size_of::<T>()).cast();
            Some(&*ptr)
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            let slice = slice_from_raw_parts_mut(self.block.cast::<T>(), self.len);
            std::ptr::drop_in_place(slice);
            self.len = 0;
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn iter<'a>(&'a self) -> TypedArenaIter<'a, T> {
        TypedArenaIter {
            inner: self,
            cursor: 0,
        }
    }
}


pub struct TypedArenaIter<'a, T> {
    inner: &'a TypedArena<T>,
    cursor: usize,
}

impl<'a, T> Iterator for TypedArenaIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.inner.get(self.cursor);
        self.cursor += 1;
        res
    }
}
