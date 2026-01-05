use std::alloc;
use std::mem;
use std::ptr::NonNull;
use std::marker::PhantomData;

use crate::arena::ptr::ArenaPtr;

/// This is equivalent to a Vec<Box\<dyn Any\>>, but without the need to Box the element on insertion.
/// Performance wise, with naive duration-based testing, this data structure is very competitive against std::Vec.
/// 
/// - On push, slightly slower than Vec\<T\> but much faster than Vec<Box\<dyn Any\>>, with caveat the pushes were within the reserved capacity.
/// - A normal push (and growing the capacity dynamically) should be slower than std::Vec, because currently on every realloc, only 4 more capacity is reserved with the idea of being space efficient.
/// - It's highly advised to reserve the needed capacity before use.
/// - Iterating the elements is slightly faster than Vec\<T\> and much faster than Vec<Box\<dyn Any\>>.
pub struct TypeErasedBuffer {
    pub(crate) raw: RawBuffer,
    pub(crate) len: usize,
    item_layout: alloc::Layout,
}

impl Drop for TypeErasedBuffer {
    fn drop(&mut self) {
        self.clear();
        self.raw.dealloc(self.item_layout);
    }
}

impl TypeErasedBuffer {
    pub fn new<T>() -> Self {
        let item_layout = alloc::Layout::new::<T>();

        Self {
            raw: RawBuffer::new::<T>(),
            len: 0,
            item_layout,
        }
    }

    #[inline]
    pub fn with_capacity<T>(capacity: usize) -> Self {
        let item_layout = alloc::Layout::new::<T>();
        let block = RawBuffer::with_capacity::<T>(capacity, item_layout);

        Self {
            raw: block,
            len: 0,
            item_layout,
        }
    }

    pub fn as_slice<T>(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(self.raw.cast::<T>().cast_const(), self.len)
        }
    }

    pub fn as_slice_mut<T>(&mut self) -> &mut[T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.raw.cast::<T>(), self.len)
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn capacity(&self) -> usize {
        self.raw.capacity
    }

    #[inline(always)]
    /// Safety: you have to ensure buffer is already initialized or the number of elements are within [`capacity`](Self::capacity) - 1
    unsafe fn push_unchecked<T>(&mut self, data: T) -> *mut T {
        let raw = unsafe { self.raw.push(data, self.len) };
        self.len += 1;
        raw
    }

    /// # Safety
    /// This method assumes that buffer is already initialized via [`with_capacity`](Self::with_capacity).
    /// If you provided zero capacity on initialization, first push will return error.
    /// Since there will be no reallocation, it's safe to return the pointer to the allocated data.
    pub fn push_within_capacity<T>(&mut self, data: T) -> Result<ArenaPtr<T>, Error> {
        self.check()?;
        let raw = unsafe { self.push_unchecked(data) };

        Ok(ArenaPtr::new(raw))
    }

    #[inline(always)]
    const fn check(&self) -> Result<(), Error> {
        if self.raw.capacity == 0 {
            return Err(Error::Uninitialized);
        } else if self.len == self.raw.capacity {
            return Err(Error::MaxCapacityReached);
        } else {
            Ok(())
        }
    }

    pub fn push<T>(&mut self, data: T) {
        if let Err(err) = self.check() {
            let new_capacity = match err {
                Error::MaxCapacityReached => self.raw.capacity + 4,
                Error::Uninitialized => 4,
            };

            self.raw.grow::<T>(self.item_layout, new_capacity);
        }

        unsafe {
            self.push_unchecked(data);
        }
    }

    pub(crate) fn clear(&mut self) {
        self.raw.clear(self.len);
        self.len = 0;
    }

    #[inline(always)]
    pub const unsafe fn get_unchecked_raw(&self, index: usize) -> *mut u8 {
        unsafe {
            self.raw.get_raw(index * self.item_layout.size())
        }
    }

    #[inline(always)]
    pub const unsafe fn get_unchecked<'a, T>(&'a self, index: usize) -> &'a T {
        unsafe {
            &*self.get_unchecked_raw(index).cast()
        }
    }

    pub const fn get<'a, T>(&'a self, index: usize) -> Option<&'a T> {
        if index >= self.len { return None }

        unsafe {
            Some(self.get_unchecked(index))
        }
    }

    #[inline(always)]
    pub const unsafe fn get_unchecked_mut<'a, T>(&'a mut self, index: usize) -> &'a mut T {
        unsafe {
            &mut *self.get_unchecked_raw(index).cast()
        }
    }

    pub const fn get_mut<'a, T>(&'a mut self, index: usize) -> Option<&'a mut T> {
        if index >= self.len { return None }

        unsafe {
            Some(self.get_unchecked_mut(index))
        }
    }

    #[inline(always)]
    fn swr<R>(&mut self, index: usize, f: impl FnOnce(*mut u8) -> R) -> Option<R> {
        if self.len > 0 {
            unsafe {
                let last_index = self.len - 1;
                let ptr = self.raw.swap_remove_or_pop(index, last_index, self.item_layout.size());
                self.len -= 1;
                return Some(f(ptr));
            }
        }

        None
    }

    pub fn swap_remove<T>(&mut self, index: usize) -> Option<T> {
        self.swr::<T>(index, |raw| unsafe { raw.cast::<T>().read() })
    }

    pub fn swap_remove_and_drop<T>(&mut self, index: usize) {
        self.swr::<()>(index, |raw| unsafe { raw.cast::<T>().drop_in_place() });
    }

    pub fn pop<T>(&mut self) -> Option<T> {
        if self.len > 0 {
            unsafe {
                let last_index = self.len - 1;
                let ptr = self.raw.swap_remove_or_pop(last_index, last_index, self.item_layout.size());
                self.len -= 1;
                return Some(ptr.cast::<T>().read());
            }
        }

        None
    }

    pub fn iter<'a, T>(&'a self) -> Iter<'a, T> {
        Iter::new(self.raw.cast::<T>(), self.len())
    }

    pub fn iter_mut<'a, T>(&'a mut self) -> IterMut<'a, T> {
        IterMut::new(self.raw.cast::<T>(), self.len())
    }
}

/*
#########################################################
#
# UnmanagedBuffer
#
#########################################################
*/

/// Similar with [`TypeErasedBuffer`] but this buffer is not keeping track of how many items have been allocated.
/// This is done this way because usually this buffer will be paired with a Vec\<EntityId\> for example
/// 
/// # Safety
/// - You have to manually keep track on the amount of the items you have pushed, so you can
/// - Manually [`clear`](UnmanagedBuffer::clear), by providing the number of elements, before dropping this.
/// - Getters method are unchecked, because the caller will need to do the check beforehand (ie: check if a ComponentStorage contains an Entity).
///
/// Not calling clear before dropping may potentially cause a memory leak
pub struct UnmanagedBuffer {
    pub(crate) raw: RawBuffer,
    item_layout: alloc::Layout,
}

impl Drop for UnmanagedBuffer {
    fn drop(&mut self) {
        self.raw.dealloc(self.item_layout);
    }
}

impl UnmanagedBuffer {
    pub const fn new<T>() -> Self {
        Self {
            raw: RawBuffer::new::<T>(),
            item_layout: alloc::Layout::new::<T>(),
        }
    }

    #[inline]
    pub fn with_capacity<T>(capacity: usize) -> Self {
        let item_layout = alloc::Layout::new::<T>();
        let raw = RawBuffer::with_capacity::<T>(capacity, item_layout);

        Self {
            raw,
            item_layout,
        }
    }
    #[inline(always)]
    pub const unsafe fn get_unchecked_raw(&self, index: usize) -> *mut u8 {
        unsafe {
            self.raw.get_raw(index * self.item_layout.size())
        }
    }

    pub const unsafe fn get_unchecked<'a, T>(&'a self, index: usize) -> &'a T {
        unsafe {
            &*self.get_unchecked_raw(index).cast()
        }
    }

    pub const unsafe fn get_unchecked_mut<'a, T>(&'a mut self, index: usize) -> &'a mut T {
        unsafe {
            &mut *self.get_unchecked_raw(index).cast()
        }
    }

    #[inline(always)]
    /// Safety: you have to ensure buffer is already initialized or the number of elements are within [`capacity`](Self::capacity) - 1
    unsafe fn push_unchecked<T>(&mut self, data: T, offset: usize) -> *mut T {
        let raw = unsafe { self.raw.push(data, offset) };
        raw
    }

    /// # Safety
    /// This method assumes that buffer is already initialized via [`with_capacity`](Self::with_capacity).
    /// So it's safe to return the pointer to the allocated data, because there's no reallocation that will cause the pointer to be invalid.
    pub fn push_within_capacity<T>(&mut self, data: T, offset: usize) -> Result<ArenaPtr<T>, Error> {
        self.check(offset)?;
        let raw = unsafe { self.push_unchecked(data, offset) };

        Ok(ArenaPtr::new(raw))
    }

    #[inline(always)]
    const fn check(&self, offset: usize) -> Result<(), Error> {
        if self.raw.capacity == 0 {
            return Err(Error::Uninitialized);
        } else if offset == self.raw.capacity {
            return Err(Error::MaxCapacityReached);
        } else {
            Ok(())
        }
    }

    pub fn push<T>(&mut self, data: T, offset: usize) {
        if let Err(err) = self.check(offset) {
            let new_capacity = match err {
                Error::MaxCapacityReached => self.raw.capacity + 4,
                Error::Uninitialized => 4,
            };

            self.raw.grow::<T>(self.item_layout, new_capacity);
        }

        unsafe {
            self.push_unchecked(data, offset);
        }
    }

    /// # Safety
    /// The end bound must be provided: exclusive (1..3), or inclusive (..=2).
    /// Unbounded end-bound (4..) will panic because the length of the buffer is unknown
    pub fn as_slice<T>(&self, range: impl std::ops::RangeBounds<usize>) -> &[T] {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&val) => val,
            std::ops::Bound::Excluded(_) => unreachable!(),
            std::ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            std::ops::Bound::Included(&val) => val + 1,
            std::ops::Bound::Excluded(&val) => val,
            std::ops::Bound::Unbounded => panic!(
                "Buffer has no information on the length of elements,
                must provide the end bound of the range"
            ),
        };

        unsafe {
            let len = if end == start {
                1
            } else {
                end.saturating_sub(start)
            };

            let data = self.raw.ptr.cast::<T>().add(start).as_ptr().cast_const();

            &*std::ptr::slice_from_raw_parts(data, len)
        }
    }

    #[inline(always)]
    pub fn swap_remove<R>(&mut self, index: usize, len: usize) -> Option<R> {
        if len > 0{
            unsafe {
                let last_index = len - 1;
                let ptr = self.raw.swap_remove_or_pop(index, last_index, self.item_layout.size());
                return Some(ptr.cast::<R>().read());
            }
        }

        None
    }

    pub fn pop<T>(&mut self, len: usize) -> Option<T> {
        if len > 0 {
            unsafe {
                let last_index = len - 1;
                let ptr = self.raw.swap_remove_or_pop(last_index, last_index, self.item_layout.size());
                return Some(ptr.cast::<T>().read());
            }
        }

        None
    }

    pub fn clear(&mut self, len: usize) {
        self.raw.clear(len);
    }

    pub fn iter<T>(&self, len: usize) -> Iter<'_, T> {
        Iter::new(self.raw.cast::<T>(), len)
    }

    pub fn iter_mut<T>(&mut self, len: usize) -> IterMut<'_, T> {
        IterMut::new(self.raw.cast::<T>(), len)
    }
}

/*
#########################################################
#
# RawBuffer
#
#########################################################
*/

pub(crate) struct RawBuffer {
    pub(crate) ptr: NonNull<u8>,
    pub(crate) capacity: usize,
    drop: Option<unsafe fn(*mut u8, usize)>,
}

impl RawBuffer {
    pub(crate) const fn new<T>() -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity: if size_of::<T>() == 0 { usize::MAX } else { 0 },
            drop: None,
        }
    }

    #[inline(always)]
    pub(crate) fn with_capacity<T>(capacity: usize, item_layout: alloc::Layout) -> Self {
        let mut this = Self::new::<T>();

        if capacity > 0 && this.capacity == 0 {
            this.grow::<T>(item_layout, capacity);
        }

        this
    }

    pub(crate) fn grow<T>(&mut self, item_layout: alloc::Layout, new_capacity: usize) {
        let size_t = item_layout.size();
        let align_t = item_layout.align();
        let new_size = size_t * new_capacity;

        let (layout, ptr) = if self.capacity == 0 {
            let layout = alloc::Layout::from_size_align(new_size, align_t).unwrap();
            let ptr = unsafe { alloc::alloc(layout) };

            (layout, ptr)
        } else {
            let old_size = size_t * self.capacity;
            let layout = alloc::Layout::from_size_align(old_size, align_t).unwrap();
            let ptr = unsafe { alloc::realloc(self.ptr.as_ptr(), layout, new_size) };

            (layout, ptr)
        };

        match NonNull::new(ptr) {
            Some(new) => {
                self.ptr = new;
                self.capacity = new_capacity;
            },
            None => alloc::handle_alloc_error(layout),
        }

        if self.drop.is_none() && mem::needs_drop::<T>() {
            #[inline]
            unsafe fn drop<T>(raw: *mut u8, len: usize) {
                unsafe {
                    std::ptr::slice_from_raw_parts_mut(raw.cast::<T>(), len).drop_in_place();
                }
            }

            self.drop = Some(drop::<T>)
        }
    }

    pub(crate) const fn cast<T>(&self) -> *mut T {
        self.ptr.cast::<T>().as_ptr()
    }

    #[inline(always)]
    pub(crate) const unsafe fn get_raw(&self, offset: usize) -> *mut u8 {
        unsafe {
            self.ptr.add(offset).as_ptr()
        }
    }

    #[inline(always)]
    pub(crate) const unsafe fn push<T>(&mut self, data: T, offset: usize) -> *mut T {
        if size_of::<T>() == 0 {
            unsafe {
                let ptr = self.cast::<T>();
                std::ptr::write(ptr, data);
                return ptr;
            }
        }

        unsafe {
            let raw = self.cast::<T>().add(offset);
            std::ptr::write(raw, data);
            raw
        }
    }

    #[inline(always)]
    /// this method already handle if index is equal to last_index or not -> swapping or popping
    pub(crate) unsafe fn swap_remove_or_pop(&mut self, index: usize, last_index: usize, size_t: usize) -> *mut u8 {
        if size_t == 0 {
            let ptr = std::ptr::without_provenance_mut(self.ptr.as_ptr() as usize + index);
            return ptr;
        }

        unsafe {
            let last = self.get_raw(last_index * size_t);

            if index < last_index {
                let to_remove = self.get_raw(index * size_t);
                std::ptr::swap_nonoverlapping(to_remove, last, size_t);
            }

            last
        }
    }

    #[inline(always)]
    pub(crate) fn clear(&mut self, len: usize) {
        let drop = self.drop.take();
        if let Some(drop) = drop {
            unsafe {
                drop(self.ptr.as_ptr(), len)
            }
        }
        self.drop = drop;
    }

    #[inline(always)]
    pub(crate) fn dealloc(&mut self, item_layout: alloc::Layout) {
        let size_t = item_layout.size();

        if self.capacity > 0 && size_t > 0 {
            unsafe {
                let size = size_t * self.capacity;
                let align = item_layout.align();
                let layout = alloc::Layout::from_size_align_unchecked(size, align);
                alloc::dealloc(self.ptr.as_ptr(), layout);
            }
        }
    }
}

/*
#########################################################
#
# Iterator
#
#########################################################
*/

pub struct Iter<'a, T> {
    start: *mut T,
    end: *mut T,
    marker: PhantomData<&'a [T]>,
}

impl<'a, T> Iter<'a, T> {
    pub fn new(start: *mut T, len: usize) -> Self {
        let end = if size_of::<T>() == 0 {
            std::ptr::without_provenance_mut(start as usize + len)
        } else {
            unsafe { start.add(len) }
        };

        Self {
            start,
            end,
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if size_of::<T>() == 0 {
                    self.start = std::ptr::without_provenance_mut(self.start as usize + 1);
                    Some(&*NonNull::<T>::dangling().as_ptr())
                } else {
                    let next = self.start;
                    self.start = next.add(1);
                    Some(&*next)
                }
            }
        }
    }
}

impl<'a, T: 'a> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if size_of::<T>() == 0 {
                    self.end = std::ptr::without_provenance_mut(self.end as usize - 1);
                    Some(&*NonNull::<T>::dangling().as_ptr())
                } else {
                    self.end = self.end.sub(1);
                    Some(&*self.end)
                }
            }
        }
    }
}

pub struct IterMut<'a, T> {
    start: *mut T,
    end: *mut T,
    marker: PhantomData<&'a mut [T]>,
}

impl<'a, T> IterMut<'a, T> {
    pub fn new(start: *mut T, len: usize) -> Self {
        let end = if size_of::<T>() == 0 {
            std::ptr::without_provenance_mut(start as usize + len)
        } else {
            unsafe { start.add(len) }
        };

        Self {
            start,
            end,
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if size_of::<T>() == 0 {
                    self.start = std::ptr::without_provenance_mut(self.start as usize + 1);
                    Some(&mut *NonNull::<T>::dangling().as_ptr())
                } else {
                    let next = self.start;
                    self.start = next.add(1);
                    Some(&mut *next)
                }
            }
        }
    }
}

impl<'a, T: 'a> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if size_of::<T>() == 0 {
                    self.end = std::ptr::without_provenance_mut(self.end as usize - 1);
                    Some(&mut *NonNull::<T>::dangling().as_ptr())
                } else {
                    self.end = self.end.sub(1);
                    Some(&mut *self.end)
                }
            }
        }
    }
}

/*
#########################################################
#
# Errors
#
#########################################################
*/

#[derive(Debug)]
pub enum Error {
    MaxCapacityReached,
    Uninitialized,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

/*
#########################################################
#                                                       #
#                         TEST                          #
#                                                       #
#########################################################
*/

#[cfg(test)]
mod buffer_test {
    use super::*;

    #[derive(Debug)]
    struct Obj {
        name: String,
        age: u32,
    }

    impl Drop for Obj {
        fn drop(&mut self) {
            println!("dropping {} aged {}", self.name, self.age)
        }
    }

    #[test]
    fn push_and_get() {
        let mut buffer = TypeErasedBuffer::with_capacity::<Obj>(1);
        assert!(buffer.raw.drop.is_some());

        let balo = Obj { name: "Balo".to_string(), age: 69 };
        let nunez = Obj { name: "Nunez".to_string(), age: 888 };
    
        buffer.push(balo);
        buffer.push(nunez);

        let get_mut = buffer.get_mut::<Obj>(1);
        assert!(get_mut.is_some());

        get_mut.unwrap().age = 666;
    
        let get = buffer.get::<Obj>(1);
        assert!(get.is_some_and(|obj| obj.age == 666));
    
        println!("{:?}", get.unwrap());
        println!("quitting");
    }

    #[test]
    fn remove() {
        let mut buffer = TypeErasedBuffer::with_capacity::<Obj>(5);

        for i in 0..5 {
            buffer.push(Obj { name: i.to_string(), age: i as _ });
        }

        let to_remove = 1;
        let removed = buffer.swap_remove::<Obj>(to_remove);
        assert!(removed.is_some());

        let removed = removed.unwrap();
        assert!(removed.age == to_remove as _);
    }

    #[test]
    fn zst() {
        const CAP: usize = 10;
        #[derive(Debug, PartialEq)] struct Zst;

        let mut buffer = TypeErasedBuffer::new::<Zst>();
        for _ in 0..CAP {
            buffer.push(Zst);
        }

        let first = buffer.get::<Zst>(0);
        let second = buffer.get::<Zst>(1);

        assert_eq!(first, second);

        buffer.iter::<Zst>().for_each(|zst| println!("{zst:?}"));

        let removed = buffer.pop::<Zst>();
        assert!(removed.is_some());
    }

    #[test]
    fn unmanaged() {
        let mut buffer = UnmanagedBuffer::with_capacity::<&'static str>(2);
        buffer.push("Balo", 0);
        buffer.push("Nunez", 1);

        let slice = buffer.as_slice::<&str>(..2);
        println!("{slice:?}");

        let slice = buffer.as_slice::<&str>(1..1);
        println!("{slice:?}");

        let slice = buffer.as_slice::<&str>(2..1);
        println!("{slice:?}");

        buffer.clear(2);
        drop(buffer);
    }
}
