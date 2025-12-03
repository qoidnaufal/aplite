use std::alloc;
use std::mem;
use std::ptr::NonNull;
use std::cell::UnsafeCell;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum Error {
    MaxCapacityReached,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

/// Type erased data storage.
/// This is slower than normal `Vec<T>`, but faster than `Vec<Box<dyn Any>>`.
/// However, this has double the size (48) compared to a normal Vec (24) which comes from the need to carry additional informations.
/// Removal of an element is only supported via swap remove to preserve the contiguousness of the data.
pub struct CpuBuffer {
    block: RawCpuBuffer,
    len: usize,
    item_layout: alloc::Layout,
}

impl Drop for CpuBuffer {
    fn drop(&mut self) {
        self.clear();
        self.block.dealloc(self.item_layout);
    }
}

impl CpuBuffer {
    pub fn new<T>() -> Self {
        Self::with_capacity::<T>(0)
    }

    #[inline]
    pub fn with_capacity<T>(capacity: usize) -> Self {
        let item_layout = alloc::Layout::new::<T>();
        let block = RawCpuBuffer::with_capacity::<T>(capacity, item_layout);

        Self {
            block,
            len: 0,
            item_layout,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.block.capacity
    }

    pub fn push_no_realloc<T>(&mut self, data: T) -> Result<(), Error> {
        if self.len == self.block.capacity {
            return Err(Error::MaxCapacityReached)
        }

        self.block.push_unmanaged(data, self.len);
        self.len += 1;

        Ok(())
    }

    pub fn push<T>(&mut self, data: T) {
        self.realloc_if_needed();
        self.block.push_unmanaged(data, self.len);
        self.len += 1;
    }

    fn realloc_if_needed(&mut self) {
        if self.len == self.block.capacity {
            let new_capacity = self.block.capacity + 4;
            self.block.realloc_unmanaged(self.item_layout, new_capacity);
        }
    }

    pub fn get<'a, T>(&'a self, index: usize) -> Option<&'a T> {
        if index >= self.len { return None }

        unsafe {
            let raw = self.block.get_raw(index * size_of::<T>());
            Some(&*raw.cast::<T>())
        }
    }

    pub fn get_mut<'a, T>(&'a mut self, index: usize) -> Option<&'a mut T> {
        if index >= self.len { return None }

        unsafe {
            let raw = self.block.get_raw(index * size_of::<T>());
            Some(&mut *raw.cast::<T>())
        }
    }
    
    pub fn get_cell<T>(&self, index: usize) -> Option<&UnsafeCell<T>> {
        if index >= self.len { return None }
       
        unsafe {
            let raw = self.block.get_raw(index * size_of::<T>());
            let ptr = raw.cast::<UnsafeCell<T>>();
            Some(&*ptr)
        }
    }

    pub fn swap_remove<T>(&mut self, index: usize) -> Option<T> {
        index.lt(&self.len).then_some(unsafe {
            let last_index = self.len - 1;
            let ptr = self.block.swap_remove_unmanaged::<T>(index, last_index);
            self.len -= 1;
            ptr.read()
        })
    }

    pub fn swap_remove_and_drop<T>(&mut self, index: usize) {
        if index < self.len {
            unsafe {
                let last_index = self.len - 1;
                let ptr = self.block.swap_remove_unmanaged::<T>(index, last_index);
                self.len -= 1;
                std::ptr::drop_in_place(ptr);
            }
        }
    }

    pub fn iter<'a, T>(&'a self) -> Iter<'a, T> {
        Iter::new(self)
    }

    pub fn iter_mut<'a, T>(&'a mut self) -> IterMut<'a, T> {
        IterMut::new(self)
    }

    pub fn iter_cell<'a, T>(&'a self) -> UnsafeCellIter<'a, T> {
        UnsafeCellIter::new(self)
    }

    pub(crate) fn clear(&mut self) {
        self.block.clear(self.len);
        self.len = 0;
    }
}

pub(crate) struct RawCpuBuffer {
    block: NonNull<u8>,
    pub(crate) capacity: usize,
    drop: Option<unsafe fn(*mut u8, usize)>,
}

impl RawCpuBuffer {
    #[inline(always)]
    pub(crate) fn with_capacity<T>(capacity: usize, item_layout: alloc::Layout) -> Self {
        #[inline]
        unsafe fn drop<T>(raw: *mut u8, len: usize) {
            unsafe {
                let ptr = raw.cast::<T>();
                for i in 0..len {
                    std::ptr::drop_in_place(ptr.add(i));
                }
            }
        }

        if capacity == 0 {
            Self {
                block: NonNull::dangling(),
                capacity,
                drop: mem::needs_drop::<T>().then_some(drop::<T>),
            }
        } else {
            unsafe {
                let layout = alloc::Layout::from_size_align_unchecked(
                    item_layout.size() * capacity,
                    item_layout.align()
                );
                let raw = std::alloc::alloc(layout);

                Self {
                    block: NonNull::new(raw).unwrap_or_else(|| alloc::handle_alloc_error(layout)),
                    capacity,
                    drop: mem::needs_drop::<T>().then_some(drop::<T>),
                }
            }
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn get_raw(&self, offset: usize) -> *mut u8 {
        unsafe {
            self.block.add(offset).as_ptr()
        }
    }

    #[inline(always)]
    pub(crate) fn push_unmanaged<T>(&mut self, data: T, len: usize) {
        // let layout = alloc::Layout::new::<T>();

        // let size = layout.size();
        // let align = layout.align();

        // unsafe {
        //     let raw = self.block.add(len * size);
        //     let aligned = raw.align_offset(align);
        //     let ptr = raw.add(aligned).as_ptr();
        //     std::ptr::write(ptr.cast::<T>(), data);
        // }

        unsafe {
            let raw = self.block.add(len * size_of::<T>()).as_ptr();
            std::ptr::write(raw.cast::<T>(), data);
        }
    }

    #[inline(always)]
    pub(crate) fn realloc_unmanaged(&mut self, item_layout: alloc::Layout, new_capacity: usize) {
        unsafe {
            let new_size = item_layout.size() * new_capacity;
            let new_block = alloc::realloc(self.block.as_ptr(), item_layout, new_size);

            self.block = NonNull::new_unchecked(new_block);
            self.capacity = new_capacity;
        }
    }

    #[inline(always)]
    #[must_use]
    pub(crate) unsafe fn swap_remove_unmanaged<T>(&mut self, index: usize, last_index: usize) -> *mut T {
        let size_t = size_of::<T>();
        unsafe {
            let last = self.get_raw(last_index * size_t).cast::<T>();

            if index < last_index {
                let to_remove = self.get_raw(index * size_t).cast::<T>();
                std::ptr::swap_nonoverlapping(to_remove, last, 1);
            }

            last
        }
    }

    #[inline(always)]
    pub(crate) fn clear(&mut self, len: usize) {
        let drop = self.drop.take();
        if let Some(drop) = drop {
            unsafe {
                drop(self.block.as_ptr(), len)
            }
        }
        self.drop = drop;
    }

    #[inline(always)]
    pub(crate) fn dealloc(&mut self, item_layout: alloc::Layout) {
        unsafe {
            let size = item_layout.size() * self.capacity;
            let align = item_layout.align();
            let layout = alloc::Layout::from_size_align_unchecked(size, align);
            alloc::dealloc(self.block.as_ptr(), layout);
        }
    }
}

pub struct Iter<'a, T> {
    source: &'a CpuBuffer,
    next: usize,
    marker: PhantomData<T>,
}

impl<'a, T> Iter<'a, T> {
    fn new(source: &'a CpuBuffer) -> Self {
        Self {
            source,
            next: 0,
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.source
            .get::<T>(self.next)
            .inspect(|_| self.next += 1)
    }
}

pub struct IterMut<'a, T> {
    source: &'a mut CpuBuffer,
    next: usize,
    marker: PhantomData<T>,
}

impl<'a, T> IterMut<'a, T> {
    fn new(source: &'a mut CpuBuffer) -> Self {
        Self {
            source,
            next: 0,
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.source
            .get_cell::<T>(self.next)
            .inspect(|_| self.next += 1)
            .map(|cell| unsafe { &mut * cell.get() })
    }
}

pub struct UnsafeCellIter<'a, T> {
    source: &'a CpuBuffer,
    next: usize,
    marker: PhantomData<UnsafeCell<T>>,
}

impl<'a, T> UnsafeCellIter<'a, T> {
    fn new(source: &'a CpuBuffer) -> Self {
        Self {
            source,
            next: 0,
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for UnsafeCellIter<'a, T> {
    type Item = &'a UnsafeCell<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.source
            .get_cell::<T>(self.next)
            .inspect(|_| self.next += 1)
    }
}

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
        let mut ba = CpuBuffer::with_capacity::<Obj>(1);
        assert!(ba.block.drop.is_some());

        let balo = Obj { name: "Balo".to_string(), age: 69 };
        let nunez = Obj { name: "Nunez".to_string(), age: 888 };
    
        ba.push(balo);
        ba.push(nunez);
    
        let get = ba.get_cell::<Obj>(1).map(|cell| unsafe {
            let raw = cell.get();
            let this = &mut *raw;
            this.age = 0;
            &*raw
        });

        assert!(get.is_some_and(|obj| obj.age == 0));
    
        println!("{:?}", get.unwrap());
        println!("quitting");
    }

    #[test]
    fn remove() {
        let mut ba = CpuBuffer::with_capacity::<Obj>(5);

        for i in 0..5 {
            ba.push(Obj { name: i.to_string(), age: i as _ });
        }

        let to_remove = 1;
        let removed = ba.swap_remove::<Obj>(to_remove);
        assert!(removed.is_some());

        let removed = removed.unwrap();
        assert!(removed.age == to_remove as _);
    }

    #[test]
    fn iter() {
        let mut ba = CpuBuffer::with_capacity::<Obj>(5);

        for i in 0..5 {
            ba.push(Obj { name: i.to_string(), age: i as _ });
        }

        let iter = ba.iter_cell::<Obj>();
        iter.for_each(|cell| unsafe {
            let obj = &mut *cell.get();
            obj.age = 0;
        });

        let mut iter2 = ba.iter_cell::<Obj>();
        assert!(iter2.all(|cell| unsafe {
            let obj = &*cell.get();
            obj.age == 0
        }))
    }

    #[test]
    fn zst() {
        const CAP: usize = 2;
        #[derive(Debug, PartialEq)]
        struct Zst;
        let mut ba = CpuBuffer::with_capacity::<Zst>(CAP);
        for _ in 0..CAP {
            ba.push(Zst);
        }

        let first = ba.get::<Zst>(0);
        let second = ba.get::<Zst>(1);

        assert_eq!(first, second);
    }

    #[test]
    fn speed() {
        struct NewObj {
            _name: String,
            _age: usize,
            _addr: usize,
        }

        const NUM: usize = 1024 * 1024;

        let mut ba = CpuBuffer::with_capacity::<NewObj>(NUM);
        let now = std::time::Instant::now();
        for i in 0..NUM {
            ba.push(NewObj { _name: i.to_string(), _age: i, _addr: i });
        }
        println!("buffer push time for {NUM} objects: {:?}", now.elapsed());

        let mut vec: Vec<NewObj> = Vec::with_capacity(NUM);
        let now = std::time::Instant::now();
        for i in 0..NUM {
            vec.push(NewObj { _name: i.to_string(), _age: i, _addr: i });
        }
        println!("vec push time for {NUM} objects: {:?}", now.elapsed());

        let mut vec: Vec<Box<dyn std::any::Any>> = Vec::with_capacity(NUM);
        let now = std::time::Instant::now();
        for i in 0..NUM {
            vec.push(Box::new(NewObj { _name: i.to_string(), _age: i, _addr: i }));
        }
        println!("Vec<Box<dyn Any>> push time for {NUM} objects: {:?}", now.elapsed());
    }
}
