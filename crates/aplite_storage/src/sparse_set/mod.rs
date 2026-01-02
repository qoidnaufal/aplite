pub(crate) mod indices;

use std::alloc;
use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::buffer::{RawBuffer, Error};
use crate::arena::ptr::ArenaPtr;
use crate::sparse_set::indices::{SparseIndices, SparsetKey};

pub struct SparseSet<K, V> {
    pub(crate) raw: RawBuffer,
    pub(crate) keys: Vec<K>,
    pub(crate) indexes: SparseIndices,
    marker: PhantomData<V>
}

impl<K, V> Drop for SparseSet<K, V> {
    fn drop(&mut self) {
        self.clear();
        self.raw.dealloc(Self::LAYOUT);
    }
}

impl<K, V> SparseSet<K, V> {
    const LAYOUT: alloc::Layout = alloc::Layout::new::<V>();
    const SIZE_V: usize = size_of::<V>();
    const ALIGN_V: usize = align_of::<V>();

    pub fn keys(&self) -> &[K] {
        &self.keys
    }

    pub fn values<'a>(&'a self) -> &'a [V] {
        unsafe {
            &*std::ptr::slice_from_raw_parts(
                self.raw.block.cast::<V>().as_ptr().cast_const(),
                self.len()
            )
        }
    }

    /// this is unsafe, because removing element(s) from this slice should be done from [`SparseSet::swap_remove`]
    pub unsafe fn values_mut<'a>(&'a mut self) -> &'a mut [V] {
        unsafe {
            &mut *std::ptr::slice_from_raw_parts_mut(
                self.raw.block.cast::<V>().as_ptr(),
                self.len()
            )
        }
    }

    pub fn clear(&mut self) {
        if self.keys.len() > 0 {
            self.indexes.clear();
            self.raw.clear(self.keys.len());
            self.keys.clear();
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.raw.capacity
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
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            raw: RawBuffer::with_capacity::<V>(capacity, Self::LAYOUT),
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
                let ptr = self.raw.get_raw(index * Self::SIZE_V).cast();
                NonNull::new(ptr)
            })
    }

    #[inline(always)]
    pub unsafe fn get_unchecked(&self, key: K) -> &V {
        unsafe {
            let index = self.indexes.get_index_unchecked(key);
            &*self.raw.get_raw(index * Self::SIZE_V).cast()
        }
    }

    #[inline(always)]
    pub fn get(&self, key: K) -> Option<&V> {
        self.indexes
            .get_index(key)
            .map(|index| unsafe {
                &*self.raw
                    .get_raw(index * Self::SIZE_V)
                    .cast()
            })
    }

    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self, key: K) -> &mut V {
        unsafe {
            let index = self.indexes.get_index_unchecked(key);
            &mut *self.raw.get_raw(index * Self::SIZE_V).cast()
        }
    }

    #[inline(always)]
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.indexes
            .get_index(key)
            .map(|index| unsafe {
                &mut *self.raw
                    .get_raw(index * Self::SIZE_V)
                    .cast()
            })
    }

    #[inline(always)]
    pub fn get_data_index(&self, key: K) -> Option<usize> {
        self.indexes.get_index(key)
    }

    #[inline(always)]
    /// Safety: you have to ensure len < capacity, and the entity does not existed yet within this sparse_set
    pub unsafe fn insert_unchecked(&mut self, key: K, value: V, len: usize) -> ArenaPtr<V> {
        self.indexes.set_index(key, len);
        let ptr = unsafe { ArenaPtr::new(self.raw.push(value, len)) };
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

        if self.raw.capacity == 0 {
            return Err(Error::Uninitialized)
        }

        let len = self.len();
        if len >= self.raw.capacity {
            return Err(Error::MaxCapacityReached)
        }

        Ok(unsafe { self.insert_unchecked(key, value, len) })
    }

    #[inline(always)]
    pub fn insert(&mut self, key: K, value: V) -> ArenaPtr<V> {
        if let Some(exist) = self.get_mut(key) {
            *exist = value;
            return ArenaPtr::new(exist);
        }

        if self.raw.capacity == 0 {
            self.raw.initialize(4, Self::SIZE_V, Self::ALIGN_V);
        }

        let len = self.len();
        if len >= self.raw.capacity {
            self.raw.realloc(Self::LAYOUT, len + 4);
        }

        unsafe { self.insert_unchecked(key, value, len) }
    }

    #[inline(always)]
    pub fn get_or_insert_with(&mut self, key: K, new: impl FnOnce() -> V) -> ArenaPtr<V> {
        if let Some(exist) = self.get_mut(key) {
            return ArenaPtr::new(exist);
        }

        if self.raw.capacity == 0 {
            self.raw.initialize(4, Self::SIZE_V, Self::ALIGN_V);
        }

        let len = self.len();
        if len >= self.raw.capacity {
            self.raw.realloc(Self::LAYOUT, len + 4);
        }

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
                self.raw
                    .swap_remove_raw(index, len - 1, Self::SIZE_V)
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
        // self.values().iter()
        Iter::new(self)
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<'_, V> {
        // unsafe {
        //     self.values_mut().iter_mut()
        // }
        IterMut::new(self)
    }
}

/*
#########################################################
#
# Iterator
#
#########################################################
*/

use crate::buffer::Base;

pub struct Iter<'a, T> {
    base: Base<T>,
    marker: PhantomData<&'a T>,
}

impl<'a, V> Iter<'a, V> {
    pub(crate) fn new<K>(source: &'a SparseSet<K, V>) -> Self {
        Self {
            base: Base::new(source.raw.block.cast::<V>().as_ptr(), source.len()),
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.base.next::<&'a T>(|raw| &*raw)
        }
    }
}

impl<'a, T: 'a> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            self.base.back::<&'a T>(|raw| &*raw)
        }
    }
}

pub struct IterMut<'a, T> {
    base: Base<T>,
    marker: PhantomData<&'a mut T>,
}

impl<'a, V> IterMut<'a, V> {
    pub(crate) fn new<K>(source: &'a mut SparseSet<K, V>) -> Self {
        Self {
            base: Base::new(source.raw.block.cast::<V>().as_ptr(), source.len()),
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.base.next::<&'a mut T>(|raw| &mut *raw)
        }
    }
}

impl<'a, T: 'a> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            self.base.back(|raw| &mut *raw)
        }
    }
}

#[cfg(test)]
mod type_erased_sparse_set {
    use super::*;
    use crate::entity::EntityId;

    #[derive(Debug, PartialEq)] struct Obj { name: String, age: u32 }

    impl Drop for Obj {
        fn drop(&mut self) {
            println!("dropped {} aged {}", self.name, self.age)
        }
    }

    impl Obj {
        fn new(name: &str, age: u32) -> Self {
            Self { name: name.to_string(), age }
        }
    }

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
}
