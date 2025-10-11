use std::alloc;
use std::marker::PhantomData;
use std::ptr::slice_from_raw_parts_mut;

pub struct Arena<T> {
    block: *mut u8,
    capacity: usize,
    len: usize,
    marker: PhantomData<T>,
}

impl<T> Drop for Arena<T> {
    fn drop(&mut self) {
        unsafe {
            let size = size_of::<T>() * self.capacity;
            let align = align_of::<T>();
            let slice = slice_from_raw_parts_mut(self.block.cast::<T>(), self.len);
            std::ptr::drop_in_place(slice);
            let layout = alloc::Layout::from_size_align_unchecked(size, align);
            alloc::dealloc(self.block, layout);
        }
    }
}

impl<T> Arena<T> {
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
        assert!(self.len <= self.capacity);
        unsafe {
            let ptr = self.block.add(self.len * size_of::<T>()).cast();
            self.len += 1;
            std::ptr::write(ptr, data);

            Ptr { ptr }
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
        if index >= self.len { panic!() }
        unsafe {
            let ptr = self.block.add(index * size_of::<T>()).cast();
            Ptr { ptr }
        }
    }

    pub fn last(&self) -> Option<&T> {
        if self.len == 0 { return None }
        unsafe {
            let ptr = self.block.add((self.len - 1) * size_of::<T>()).cast();
            Some(&*ptr)
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn iter<'a>(&'a self) -> ArenaIter<'a, T> {
        ArenaIter {
            inner: self,
            cursor: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Ptr<T: ?Sized> {
    ptr: *mut T,
}

impl<T: ?Sized> Ptr<T> {
    pub fn map<U: ?Sized>(mut self, f: impl FnOnce(&mut T) -> &mut U) -> Ptr<U> {
        Ptr {
            ptr: f(&mut self)
        }
    }
}

impl<T: ?Sized> std::ops::Deref for Ptr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.ptr
        }
    }
}

impl<T: ?Sized> std::ops::DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.ptr
        }
    }
}

pub struct ArenaIter<'a, T> {
    inner: &'a Arena<T>,
    cursor: usize,
}

impl<'a, T> Iterator for ArenaIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.inner.get(self.cursor);
        self.cursor += 1;
        res
    }
}
