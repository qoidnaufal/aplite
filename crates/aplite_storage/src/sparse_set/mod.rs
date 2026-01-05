pub(crate) mod indices;

use std::alloc;
use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::buffer::{RawBuffer, Error, Iter, IterMut};
use crate::arena::ptr::ArenaPtr;
use crate::sparse_set::indices::{SparseIndices, SparsetKey};

pub struct TypeErasedSparseSet<K> {
    pub(crate) data_buffer: RawBuffer,
    pub(crate) keys: Vec<K>,
    pub(crate) indexes: SparseIndices,
    layout: alloc::Layout,
}

pub struct SparseSet<K, V> {
    pub(crate) data_buffer: RawBuffer,
    pub(crate) keys: Vec<K>,
    pub(crate) indexes: SparseIndices,
    marker: PhantomData<V>
}

/*
#########################################################
#
# impl TypeErasedSparseSet
#
#########################################################
*/

impl<K> Drop for TypeErasedSparseSet<K> {
    fn drop(&mut self) {
        self.clear();
        self.data_buffer.dealloc(self.layout);
    }
}

impl<K> TypeErasedSparseSet<K> {
    pub fn keys(&self) -> &[K] {
        &self.keys
    }

    pub fn values<V: 'static>(&self) -> &[V] {
        unsafe {
            &*std::ptr::slice_from_raw_parts(
                self.data_buffer.ptr.cast::<V>().as_ptr().cast_const(),
                self.len()
            )
        }
    }

    pub fn values_mut<V: 'static>(&mut self) -> &mut [V] {
        unsafe {
            &mut *std::ptr::slice_from_raw_parts_mut(
                self.data_buffer.ptr.cast::<V>().as_ptr(),
                self.len()
            )
        }
    }

    pub fn capacity(&self) -> usize {
        self.data_buffer.capacity
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        let len = self.keys.len();
        if len > 0 {
            self.indexes.clear();
            self.data_buffer.clear(len);
            self.keys.clear();
        }
    }
}

impl<K: SparsetKey> TypeErasedSparseSet<K> {
    pub const fn new<V: 'static>() -> Self {
        Self {
            data_buffer: RawBuffer::new::<V>(),
            keys: Vec::new(),
            indexes: SparseIndices::new(),
            layout: alloc::Layout::new::<V>(),
        }
    }

    #[inline(always)]
    pub fn with_capacity<V: 'static>(capacity: usize) -> Self {
        let layout = alloc::Layout::new::<V>();
        Self {
            data_buffer: RawBuffer::with_capacity::<V>(capacity, layout),
            keys: Vec::with_capacity(capacity),
            indexes: SparseIndices::default(),
            layout,
        }
    }

    #[inline(always)]
    pub unsafe fn get_raw<V: 'static>(&self, key: K) -> Option<NonNull<V>> {
        self.indexes
            .get_index(key)
            .and_then(|index| unsafe {
                let ptr = self.data_buffer.get_raw(index * self.layout.size()).cast();
                NonNull::new(ptr)
            })
    }

    #[inline(always)]
    pub unsafe fn get_unchecked<V: 'static>(&self, key: K) -> &V {
        unsafe {
            let index = self.indexes.get_index_unchecked(key);
            &*self.data_buffer.get_raw(index * self.layout.size()).cast()
        }
    }

    #[inline(always)]
    pub fn get<V: 'static>(&self, key: K) -> Option<&V> {
        self.indexes
            .get_index(key)
            .map(|index| unsafe {
                &*self.data_buffer
                    .get_raw(index * self.layout.size())
                    .cast()
            })
    }

    #[inline(always)]
    pub unsafe fn get_unchecked_mut<V: 'static>(&mut self, key: K) -> &mut V {
        unsafe {
            let index = self.indexes.get_index_unchecked(key);
            &mut *self.data_buffer.get_raw(index * self.layout.size()).cast()
        }
    }

    #[inline(always)]
    pub fn get_mut<V: 'static>(&mut self, key: K) -> Option<&mut V> {
        self.indexes
            .get_index(key)
            .map(|index| unsafe {
                &mut *self.data_buffer
                    .get_raw(index * self.layout.size())
                    .cast()
            })
    }

    #[inline(always)]
    const fn check(&self, offset: usize) -> Result<(), Error> {
        if self.data_buffer.capacity == 0 {
            return Err(Error::Uninitialized);
        } else if offset == self.data_buffer.capacity {
            return Err(Error::MaxCapacityReached);
        } else {
            Ok(())
        }
    }

    fn grow_if_needed<V>(&mut self, len: usize) {
        if let Err(err) = self.check(len) {
            let new_capacity = match err {
                Error::MaxCapacityReached => self.data_buffer.capacity + 4,
                Error::Uninitialized => 4,
            };

            self.data_buffer.grow::<V>(self.layout, new_capacity);
        }
    }

    #[inline(always)]
    /// Safety: you have to ensure len < capacity, and the entity does not existed yet within this sparse_set
    pub unsafe fn insert_unchecked<V: 'static>(&mut self, key: K, value: V, len: usize) -> ArenaPtr<V> {
        self.indexes.set_index(key, len);
        let ptr = unsafe { ArenaPtr::new(self.data_buffer.push(value, len)) };
        self.keys.push(key);
        ptr
    }

    #[inline(always)]
    pub fn insert_within_capacity<V: 'static>(
        &mut self,
        key: K,
        value: V,
    ) -> Result<ArenaPtr<V>, Error> {
        if let Some(exist) = self.get_mut(key) {
            *exist = value;
            return Ok(ArenaPtr::new(exist));
        }

        let len = self.len();
        self.check(len)?;

        Ok(unsafe { self.insert_unchecked(key, value, len) })
    }

    #[inline(always)]
    pub fn insert<V: 'static>(&mut self, key: K, value: V) -> ArenaPtr<V> {
        if let Some(exist) = self.get_mut(key) {
            *exist = value;
            return ArenaPtr::new(exist);
        }

        let len = self.len();
        self.grow_if_needed::<V>(len);

        unsafe { self.insert_unchecked(key, value, len) }
    }

    #[inline(always)]
    pub fn get_or_insert_with<V: 'static>(&mut self, key: K, new: impl FnOnce() -> V) -> ArenaPtr<V> {
        if let Some(exist) = self.get_mut(key) {
            return ArenaPtr::new(exist);
        }

        let len = self.len();
        self.grow_if_needed::<V>(len);

        unsafe { self.insert_unchecked(key, new(), len) }
    }

    #[inline(always)]
    pub fn swap_remove<V: 'static>(&mut self, key: K) -> Option<V> {
        if self.is_empty() { return None; }

        let last_key = *self.keys.last().unwrap();
        let len = self.len();

        self.indexes.get_index(key).map(|index| {
            self.indexes.set_index(last_key, index);
            self.indexes.set_null(key);
            self.keys.swap_remove(index);
            
            unsafe {
                self.data_buffer
                    .swap_remove_or_pop(index, len - 1, self.layout.size())
                    .cast::<V>()
                    .read()
            }
        })
    }

    pub fn contains_key(&self, key: K) -> bool {
        self.indexes.get_index(key).is_some()
    }

    #[inline(always)]
    pub fn iter<V: 'static>(&self) -> Iter<'_, V> {
        Iter::new(self.data_buffer.cast::<V>(), self.len())
    }

    #[inline(always)]
    pub fn iter_mut<V: 'static>(&mut self) -> IterMut<'_, V> {
        IterMut::new(self.data_buffer.cast::<V>(), self.len())
    }
}

/*
#########################################################
#
# impl SparseSet
#
#########################################################
*/

impl<K, V> Drop for SparseSet<K, V> {
    fn drop(&mut self) {
        self.clear();
        self.data_buffer.dealloc(Self::LAYOUT);
    }
}

impl<K, V> SparseSet<K, V> {
    const LAYOUT: alloc::Layout = alloc::Layout::new::<V>();

    pub fn keys(&self) -> &[K] {
        &self.keys
    }

    pub fn values<'a>(&'a self) -> &'a [V] {
        unsafe {
            &*std::ptr::slice_from_raw_parts(
                self.data_buffer.ptr.cast::<V>().as_ptr().cast_const(),
                self.len()
            )
        }
    }

    /// # Safety
    /// This is unsafe, because removing element(s) from this slice should be done from [`SparseSet::swap_remove`]
    /// Only use this method to quickly iterates over the mutable slice of values
    pub unsafe fn values_mut<'a>(&'a mut self) -> &'a mut [V] {
        unsafe {
            &mut *std::ptr::slice_from_raw_parts_mut(
                self.data_buffer.ptr.cast::<V>().as_ptr(),
                self.len()
            )
        }
    }

    pub fn clear(&mut self) {
        if self.keys.len() > 0 {
            self.indexes.clear();
            self.data_buffer.clear(self.keys.len());
            self.keys.clear();
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.data_buffer.capacity
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K: SparsetKey, V: 'static> SparseSet<K, V> {
    pub const fn new() -> Self {
        Self {
            data_buffer: RawBuffer::new::<V>(),
            keys: Vec::new(),
            indexes: SparseIndices::new(),
            marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data_buffer: RawBuffer::with_capacity::<V>(capacity, Self::LAYOUT),
            keys: Vec::with_capacity(capacity),
            indexes: SparseIndices::default(),
            marker: PhantomData,
        }
    }

    #[inline(always)]
    pub unsafe fn get_raw(&self, key: K) -> Option<NonNull<V>> {
        self.indexes
            .get_index(key)
            .and_then(|index| unsafe {
                let ptr = self.data_buffer.get_raw(index * Self::LAYOUT.size()).cast();
                NonNull::new(ptr)
            })
    }

    #[inline(always)]
    pub unsafe fn get_unchecked(&self, key: K) -> &V {
        unsafe {
            let index = self.indexes.get_index_unchecked(key);
            &*self.data_buffer.get_raw(index * Self::LAYOUT.size()).cast()
        }
    }

    #[inline(always)]
    pub fn get(&self, key: K) -> Option<&V> {
        self.indexes
            .get_index(key)
            .map(|index| unsafe {
                &*self.data_buffer
                    .get_raw(index * Self::LAYOUT.size())
                    .cast()
            })
    }

    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self, key: K) -> &mut V {
        unsafe {
            let index = self.indexes.get_index_unchecked(key);
            &mut *self.data_buffer.get_raw(index * Self::LAYOUT.size()).cast()
        }
    }

    #[inline(always)]
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.indexes
            .get_index(key)
            .map(|index| unsafe {
                &mut *self.data_buffer
                    .get_raw(index * Self::LAYOUT.size())
                    .cast()
            })
    }

    #[inline(always)]
    pub fn get_data_index(&self, key: K) -> Option<usize> {
        self.indexes.get_index(key)
    }

    #[inline(always)]
    const fn check(&self, offset: usize) -> Result<(), Error> {
        if self.data_buffer.capacity == 0 {
            return Err(Error::Uninitialized);
        } else if offset == self.data_buffer.capacity {
            return Err(Error::MaxCapacityReached);
        } else {
            Ok(())
        }
    }

    fn grow_if_needed(&mut self, len: usize) {
        if let Err(err) = self.check(len) {
            let new_capacity = match err {
                Error::MaxCapacityReached => self.data_buffer.capacity + 4,
                Error::Uninitialized => 4,
            };

            self.data_buffer.grow::<V>(Self::LAYOUT, new_capacity);
        }
    }

    #[inline(always)]
    /// Safety: you have to ensure len < capacity, and the entity does not existed yet within this sparse_set
    pub unsafe fn insert_unchecked(&mut self, key: K, value: V, len: usize) -> ArenaPtr<V> {
        self.indexes.set_index(key, len);
        let ptr = unsafe { ArenaPtr::new(self.data_buffer.push(value, len)) };
        self.keys.push(key);
        ptr
    }

    #[inline(always)]
    pub fn insert_within_capacity(
        &mut self,
        key: K,
        value: V,
    ) -> Result<ArenaPtr<V>, Error> {
        if let Some(exist) = self.get_mut(key) {
            *exist = value;
            return Ok(ArenaPtr::new(exist));
        }

        let len = self.len();
        self.check(len)?;

        Ok(unsafe { self.insert_unchecked(key, value, len) })
    }

    #[inline(always)]
    pub fn insert(&mut self, key: K, value: V) -> ArenaPtr<V> {
        if let Some(exist) = self.get_mut(key) {
            *exist = value;
            return ArenaPtr::new(exist);
        }

        let len = self.len();
        self.grow_if_needed(len);

        unsafe { self.insert_unchecked(key, value, len) }
    }

    #[inline(always)]
    pub fn get_or_insert_with(&mut self, key: K, new: impl FnOnce() -> V) -> ArenaPtr<V> {
        if let Some(exist) = self.get_mut(key) {
            return ArenaPtr::new(exist);
        }

        let len = self.len();
        self.grow_if_needed(len);

        unsafe { self.insert_unchecked(key, new(), len) }
    }

    #[inline(always)]
    pub fn swap_remove(&mut self, key: K) -> Option<V> {
        if self.is_empty() { return None; }

        let last_key = *self.keys.last().unwrap();
        let len = self.len();

        self.indexes.get_index(key).map(|index| {
            self.indexes.set_index(last_key, index);
            self.indexes.set_null(key);
            self.keys.swap_remove(index);
            
            unsafe {
                self.data_buffer
                    .swap_remove_or_pop(index, len - 1, Self::LAYOUT.size())
                    .cast::<V>()
                    .read()
            }
        })
    }

    pub fn contains_key(&self, key: K) -> bool {
        self.indexes.get_index(key).is_some()
    }

    #[inline(always)]
    pub fn iter(&self) -> Iter<'_, V> {
        Iter::new(self.data_buffer.cast::<V>(), self.len())
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<'_, V> {
        IterMut::new(self.data_buffer.cast::<V>(), self.len())
    }
}

#[cfg(test)]
mod sparse_set_test {
    use super::*;
    use crate::entity::EntityId;

    #[derive(Debug, PartialEq)] struct Obj { name: String, age: u32 }

    impl Drop for Obj { fn drop(&mut self) { println!("dropped {} aged {}", self.name, self.age) } }

    impl Obj { fn new(name: &str, age: u32) -> Self { Self { name: name.to_string(), age } } }

    #[test]
    fn swap_remove_and_drop_test() -> Result<(), Error> {
        const NUM: usize = 5;
        let mut ss = SparseSet::<EntityId, Obj>::new();
        let names = ["Balo", "Nunez", "Maguirre", "Bendtner", "Haryono"];

        for i in 0..NUM {
            let obj = Obj::new(names[i], i as _);
            ss.insert(EntityId::new(i as _), obj);
        }

        let last = EntityId::new(4);
        let to_remove = EntityId::new(1);

        let removed_index = ss.indexes.get_index(to_remove);
        let prev_index = ss.indexes.get_index(last);

        let removed = ss.swap_remove(to_remove);
        assert!(removed.is_some());
        let removed = ss.get(to_remove);
        assert!(removed.is_none());

        let new_index = ss.indexes.get_index(last);
        assert_ne!(prev_index, new_index);
        assert_eq!(new_index, removed_index);

        println!("quitting");

        Ok(())
    }

    #[test]
    fn iter() {
        const NUM: usize = 5;
        let mut ss = SparseSet::<EntityId, &'static str>::new();
        let names = ["Balo", "Nunez", "Maguirre", "Bendtner", "Haryono"];

        for i in 0..NUM {
            ss.insert(EntityId::new(i as _), names[i]);
        }

        // ss.iter().for_each(|value| println!("{value}"));
        let forward = ss.iter().collect::<Box<[_]>>();

        // ss.iter().rev().for_each(|value| println!("rev . {value}"));
        let mut backward = ss.iter().rev().collect::<Box<[_]>>();
        backward.as_mut().reverse();

        assert_eq!(forward.as_ref(), backward.as_ref());
    }

    #[test]
    fn iter_type_erased() {
        const NUM: usize = 5;
        let mut ss = TypeErasedSparseSet::<EntityId>::new::<&'static str>();
        let names = ["Balo", "Nunez", "Maguirre", "Bendtner", "Haryono"];

        for i in 0..NUM {
            ss.insert(EntityId::new(i as _), names[i]);
        }

        // ss.iter().for_each(|value| println!("{value}"));
        let forward = ss.iter::<&str>().collect::<Box<[_]>>();

        // ss.iter().rev().for_each(|value| println!("rev . {value}"));
        let mut backward = ss.iter::<&str>().rev().collect::<Box<[_]>>();
        backward.as_mut().reverse();

        assert_eq!(forward.as_ref(), backward.as_ref());
    }
}
