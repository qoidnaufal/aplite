use std::alloc;
use std::mem;
use std::ptr::NonNull;
use std::num::NonZeroUsize;
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
/// This is slightly slower than normal `Vec<T>`, but faster than `Vec<Box<dyn Any>>`.
/// However, this has double the size (48) compared to a normal Vec (24) which comes from the need to carry additional informations.
/// Removal of an element is only supported via swap remove to preserve the contiguousness of the data.
pub struct UntypedArray {
    block: NonNull<u8>,
    len: usize,
    capacity: NonZeroUsize,
    item_layout: alloc::Layout,
    drop: Option<unsafe fn(*mut u8, usize)>,
}

impl Drop for UntypedArray {
    fn drop(&mut self) {
        unsafe {
            self.clear();
            let size = self.item_layout.size() * self.capacity.get();
            let align = self.item_layout.align();
            let layout = alloc::Layout::from_size_align_unchecked(size, align);
            alloc::dealloc(self.block.as_ptr(), layout);
        }
    }
}

impl UntypedArray {
    pub fn new<T>(capacity: usize) -> Self {
        #[inline]
        unsafe fn drop<T>(raw: *mut u8, len: usize) {
            unsafe {
                let ptr = raw.cast::<T>();
                for i in 0..len {
                    let to_drop = ptr.add(i);
                    std::ptr::drop_in_place(to_drop);
                }
            }
        }

        let capacity = NonZeroUsize::try_from(capacity).unwrap();
        let size = size_of::<T>();
        let align = align_of::<T>();

        unsafe {
            let layout = alloc::Layout::from_size_align_unchecked(size * capacity.get(), align);
            let raw = std::alloc::alloc(layout);

            if raw.is_null() {
                alloc::handle_alloc_error(layout);
            }

            Self {
                block: NonNull::new_unchecked(raw),
                len: 0,
                capacity,
                item_layout: alloc::Layout::from_size_align_unchecked(size, align),
                drop: mem::needs_drop::<T>().then_some(drop::<T>),
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity.get()
    }

    pub fn push_no_realloc<T>(&mut self, data: T) -> Result<(), Error> {
        let size = size_of::<T>();
        let align = align_of::<T>();
        let capacity = self.capacity.get();

        if self.len == capacity {
            return Err(Error::MaxCapacityReached)
        }

        unsafe {
            let raw = self.block.add(self.len * size);
            let aligned = raw.align_offset(align);
            let ptr = raw.add(aligned).as_ptr();
            std::ptr::write(ptr.cast::<T>(), data);
        }

        self.len += 1;

        Ok(())
    }

    pub fn push<T>(&mut self, data: T) {
        let size = size_of::<T>();
        let align = align_of::<T>();
        let capacity = self.capacity.get();

        if self.len == capacity {
            self.realloc(capacity + 1);
        }

        unsafe {
            let raw = self.block.add(self.len * size);
            let aligned = raw.align_offset(align);
            let ptr = raw.add(aligned).as_ptr();
            std::ptr::write(ptr.cast::<T>(), data);
        }

        self.len += 1;
    }

    fn realloc(&mut self, new_capacity: usize) {
        unsafe {
            let new_size = self.item_layout.size() * new_capacity;
            let new_block = alloc::realloc(self.block.as_ptr(), self.item_layout, new_size);

            self.block = NonNull::new_unchecked(new_block);
            self.capacity = NonZeroUsize::try_from(new_capacity).unwrap();
        }
    }

    #[inline(always)]
    unsafe fn get_raw<T>(&self, index: usize) -> *mut u8 {
        debug_assert!(index < self.len);
        unsafe {
            self.block.add(index * size_of::<T>()).as_ptr()
        }
    }

    pub fn get<T>(&self, index: usize) -> Option<&T> {
        if index >= self.len { return None }

        unsafe {
            let raw = self.get_raw::<T>(index);
            Some(&*raw.cast::<T>())
        }
    }

    pub fn get_mut<T>(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len { return None }

        unsafe {
            let raw = self.get_raw::<T>(index);
            Some(&mut *raw.cast::<T>())
        }
    }
    
    pub fn get_cell<T>(&self, index: usize) -> Option<&UnsafeCell<T>> {
        if index >= self.len { return None }
       
        unsafe {
            let raw = self.get_raw::<T>(index);
            let ptr = raw.cast::<UnsafeCell<T>>();
            Some(&*ptr)
        }
    }

    pub fn swap_remove<T>(&mut self, index: usize) -> Option<T> {
        if index >= self.len { return None }

        let last_index = self.len - 1;

        unsafe {
            let last = self.get_raw::<T>(last_index).cast::<T>();
            self.len -= 1;

            if index < last_index {
                let to_remove = self.get_raw::<T>(index).cast::<T>();
                std::ptr::swap_nonoverlapping(to_remove, last, 1);
            }

            Some(last.read())
        }
    }

    pub fn swap_remove_drop<T>(&mut self, index: usize) {
        if index < self.len {
            let last_index = self.len - 1;
            unsafe {
                let last = self.get_raw::<T>(last_index).cast::<T>();
                self.len -= 1;

                if index < last_index {
                    let to_remove = self.get_raw::<T>(index).cast::<T>();
                    std::ptr::swap_nonoverlapping(to_remove, last, 1);
                }

                std::ptr::drop_in_place(last);
            }
        }
    }

    pub fn iter<'a, T>(&'a self) -> Iter<'a, T> {
        Iter::new(self)
    }

    pub(crate) fn clear(&mut self) {
        let drop = self.drop.take();
        if let Some(drop) = drop {
            unsafe { drop(self.block.as_ptr(), self.len) }
            self.len = 0;
        }
        self.drop = drop;
    }
}

pub(crate) struct UnsafeUntypedArray {
    block: NonNull<u8>,
    pub(crate) capacity: NonZeroUsize,
    drop: Option<unsafe fn(*mut u8)>,
}

impl UnsafeUntypedArray {
    pub(crate) fn new<T>(capacity: usize) -> Self {
        #[inline]
        unsafe fn drop<T>(raw: *mut u8) {
            unsafe {
                let ptr = raw.cast::<T>();
                std::ptr::drop_in_place(ptr);
            }
        }

        let capacity = NonZeroUsize::try_from(capacity).unwrap();
        let size = size_of::<T>();
        let align = align_of::<T>();

        unsafe {
            let layout = alloc::Layout::from_size_align_unchecked(size * capacity.get(), align);
            let raw = std::alloc::alloc(layout);

            if raw.is_null() {
                alloc::handle_alloc_error(layout);
            }

            Self {
                block: NonNull::new_unchecked(raw),
                capacity,
                drop: mem::needs_drop::<T>().then_some(drop::<T>),
            }
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn get_raw(&self, index: usize, size_t: usize) -> *mut u8 {
        unsafe {
            self.block.add(index * size_t).as_ptr()
        }
    }

    pub(crate) fn push<T>(&mut self, data: T, len: usize) {
        let size = size_of::<T>();
        let align = align_of::<T>();

        unsafe {
            let raw = self.block.add(len * size);
            let aligned = raw.align_offset(align);
            let ptr = raw.add(aligned).as_ptr();
            std::ptr::write(ptr.cast::<T>(), data);
        }
    }

    pub(crate) fn realloc(&mut self, item_layout: alloc::Layout, new_capacity: usize) {
        unsafe {
            let new_size = item_layout.size() * new_capacity;
            let new_block = alloc::realloc(self.block.as_ptr(), item_layout, new_size);

            self.block = NonNull::new_unchecked(new_block);
            self.capacity = NonZeroUsize::try_from(new_capacity).unwrap();
        }
    }

    pub(crate) fn swap_remove_drop<T>(&mut self, index: usize, last_index: usize) {
        let size_t = size_of::<T>();
        unsafe {
            let last = self.get_raw(last_index, size_t).cast::<T>();

            if index < last_index {
                let to_remove = self.get_raw(index, size_t).cast::<T>();
                std::ptr::swap_nonoverlapping(to_remove, last, 1);
            }

            std::ptr::drop_in_place(last);
        }
    }

    pub(crate) fn clear(&mut self, len: usize, item_layout: alloc::Layout) {
        let size_t = item_layout.size();
        let drop = self.drop.take();
        if let Some(drop) = drop {
            unsafe {
                for i in 0..len {
                    let raw = self.block.add(i * size_t);
                    drop(raw.as_ptr())
                }
            }
        }
        self.drop = drop;
    }

    pub(crate) fn dealloc(&mut self, item_layout: alloc::Layout) {
        unsafe {
            let size = item_layout.size() * self.capacity.get();
            let align = item_layout.align();
            let layout = alloc::Layout::from_size_align_unchecked(size, align);
            alloc::dealloc(self.block.as_ptr(), layout);
        }
    }
}

pub struct Iter<'a, T> {
    source: &'a UntypedArray,
    next: usize,
    marker: PhantomData<UnsafeCell<T>>,
}

impl<'a, T> Iter<'a, T> {
    fn new(source: &'a UntypedArray) -> Self {
        Self {
            source,
            next: 0,
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a UnsafeCell<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.source
            .get_cell::<T>(self.next)
            .inspect(|_| self.next += 1)
    }
}

#[cfg(test)]
mod test {
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
        let mut ba = UntypedArray::new::<Obj>(1);
        assert!(ba.drop.is_some());

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
        let mut ba = UntypedArray::new::<Obj>(5);

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
        let mut ba = UntypedArray::new::<Obj>(5);

        for i in 0..5 {
            ba.push(Obj { name: i.to_string(), age: i as _ });
        }

        let iter = ba.iter::<Obj>();
        iter.for_each(|cell| unsafe {
            let obj = &mut *cell.get();
            obj.age = 0;
        });

        let mut iter2 = ba.iter::<Obj>();
        assert!(iter2.all(|cell| unsafe {
            let obj = &*cell.get();
            obj.age == 0
        }))
    }

    #[test]
    fn zst() {
        const CAP: usize = 2;
        struct Zst;
        let mut ba = UntypedArray::new::<Zst>(CAP);
        for _ in 0..CAP {
            ba.push(Zst);
        }

        unsafe {
            let first = ba.get_raw::<Zst>(0) as usize;
            let second = ba.get_raw::<Zst>(1) as usize;

            assert_eq!(first, second);
        }
    }

    #[test]
    fn speed() {
        struct NewObj {
            _name: String,
            _age: usize,
            _addr: usize,
        }

        const NUM: usize = 1024 * 1024;

        let mut ba = UntypedArray::new::<NewObj>(NUM);
        let now = std::time::Instant::now();
        for i in 0..NUM {
            ba.push(NewObj { _name: i.to_string(), _age: i, _addr: i });
        }
        println!("blob array push time for {NUM} objects: {:?}", now.elapsed());

        let mut vec: Vec<Box<dyn std::any::Any>> = Vec::with_capacity(NUM);
        let now = std::time::Instant::now();
        for i in 0..NUM {
            vec.push(Box::new(NewObj { _name: i.to_string(), _age: i, _addr: i }));
        }
        println!("vec push time for {NUM} objects: {:?}", now.elapsed());
    }
}
