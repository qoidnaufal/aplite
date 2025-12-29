use std::alloc;
use std::marker::PhantomData;

use crate::buffer::{RawBuffer, Error};
use crate::arena::ptr::ArenaPtr;
use crate::sparse_set::SparsetKey;
use crate::sparse_set::indices::SparseIndices;

/// Type erased but actually still uniform, to enable this SparseSet to be stored with other SparseSet for different type
/// If you want a data structure to hold any kind of types, use Arena.
/// # Example
/// ```ignore
/// let sparset_a = TypeErasedSparseSet::new::<String>();
/// let sparset_b = TypeErasedSparseSet::new::<usize>();
/// let mut storage = Vec::<TypeErasedSparset>::new()
/// storage.push(a);
/// storage.push(b);
/// ```
pub struct TypeErasedSparseSet {
    pub(crate) raw: RawBuffer,
    pub(crate) indexes: SparseIndices,
    item_layout: alloc::Layout,
    pub(crate) len: usize,
}

impl Drop for TypeErasedSparseSet {
    fn drop(&mut self) {
        self.clear();
        self.raw.dealloc(self.item_layout);
    }
}

impl TypeErasedSparseSet {
    pub fn new<V>() -> Self {
        Self::with_capacity::<V>(0)
    }

    #[inline(always)]
    pub fn with_capacity<V>(capacity: usize) -> Self {
        let item_layout = alloc::Layout::new::<V>();

        Self {
            indexes: SparseIndices::default(),
            raw: RawBuffer::with_capacity::<V>(capacity, item_layout),
            item_layout,
            len: 0,
        }
    }

    #[inline(always)]
    pub unsafe fn get_raw<K: SparsetKey>(&self, key: K) -> Option<*mut u8> {
        self.indexes
            .get_index(key)
            .map(|index| unsafe {
                self.raw
                    .get_raw(index * self.item_layout.size())
            })
    }

    pub unsafe fn get_unchecked<K: SparsetKey, V>(&self, key: K) -> &V {
        let index = self.indexes.0[key.index()].get().unwrap();
        unsafe {
            &*self.raw.get_raw(index * self.item_layout.size()).cast::<V>()
        }
    }

    #[inline(always)]
    pub fn get<K: SparsetKey, V>(&self, key: K) -> Option<&V> {
        self.indexes
            .get_index(key)
            .map(|index| unsafe {
                &*self.raw
                    .get_raw(index * self.item_layout.size())
                    .cast::<V>()
            })
    }

    #[inline(always)]
    pub fn get_mut<K: SparsetKey, V>(&self, key: K) -> Option<&mut V> {
        self.indexes
            .get_index(key)
            .map(|index| unsafe {
                &mut *self.raw
                    .get_raw(index * self.item_layout.size())
                    .cast::<V>()
            })
    }

    #[inline(always)]
    pub fn get_data_index<K: SparsetKey>(&self, key: K) -> Option<usize> {
        self.indexes.get_index(key)
    }

    #[inline(always)]
    /// Safety: you have to ensure len < capacity, and the entity does not existed yet within this sparse_set
    pub unsafe fn insert_unchecked<K: SparsetKey, V>(&mut self, key: K, value: V) -> ArenaPtr<V> {
        self.indexes.set_index(key, self.len);
        let ptr = unsafe { ArenaPtr::new(self.raw.push(value, self.len)) };
        self.len += 1;
        ptr
    }

    #[inline(always)]
    pub fn insert_within_capacity<K: SparsetKey, V>(
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

        if self.len >= self.raw.capacity {
            return Err(Error::MaxCapacityReached)
        }

        Ok(unsafe { self.insert_unchecked(key, value) })
    }

    #[inline(always)]
    pub fn insert<K: SparsetKey, V>(&mut self, key: K, value: V) -> ArenaPtr<V> {
        if let Some(exist) = self.get_mut(key) {
            *exist = value;
            return ArenaPtr::new(exist);
        }

        if self.raw.capacity == 0 {
            self.raw.initialize(4, self.item_layout.size(), self.item_layout.align());
        }

        if self.len >= self.raw.capacity {
            self.raw.realloc(self.item_layout, self.len + 4);
        }

        unsafe { self.insert_unchecked(key, value) }
    }

    #[inline(always)]
    pub fn get_or_insert_with<K: SparsetKey, V>(&mut self, key: K, new: impl FnOnce() -> V) -> ArenaPtr<V> {
        if let Some(exist) = self.get_mut(key) {
            return ArenaPtr::new(exist);
        }

        if self.raw.capacity == 0 {
            self.raw.initialize(4, self.item_layout.size(), self.item_layout.align());
        }

        if self.len >= self.raw.capacity {
            self.raw.realloc(self.item_layout, self.len + 4);
        }

        unsafe { self.insert_unchecked(key, new()) }
    }

    #[inline(always)]
    pub fn replace<K: SparsetKey, V>(&mut self, key: K, value: V) -> Option<ArenaPtr<V>> {
        self.get_mut(key).map(|exist| {
            *exist = value;
            ArenaPtr::new(exist)
        })
    }

    #[inline(always)]
    pub fn swap_remove<K: SparsetKey, V>(&mut self, key: K, last_key_to_swap: K) -> Option<V> {
        debug_assert!({
            self.indexes
                .get_index(last_key_to_swap)
                .is_some_and(|idx| idx == self.len - 1)
        });

        self.indexes.get_index(key).map(|index| unsafe {
            self.indexes.set_index(last_key_to_swap, index);
            self.indexes.set_null(key);
            self.len -= 1;
            
            self.raw
                .swap_remove_raw(index, self.len, self.item_layout.size())
                .cast::<V>()
                .read()
        })
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        if self.len > 0 {
            self.indexes.clear();
            self.raw.clear(self.len);
            // self.keys.clear();
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.raw.capacity
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn contains_key<K: SparsetKey>(&self, key: K) -> bool {
        self.indexes.get_index(key).is_some()
    }

    #[inline(always)]
    pub fn iter<V>(&self) -> Iter<'_, V> {
        Iter::new(self)
    }

    #[inline(always)]
    pub fn iter_mut<V>(&mut self) -> IterMut<'_, V> {
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

struct Base<T> {
    raw: *mut T,
    count: usize,
    len: usize,
}

impl<T> Base<T> {
    #[inline(always)]
    fn new(raw: *mut T, len: usize) -> Self {
        Self {
            raw,
            count: 0,
            len,
        }
    }

    #[inline(always)]
    unsafe fn next<R>(&mut self, f: impl FnOnce(*mut T) -> R) -> Option<R> {
        if self.count == self.len || self.len == 0 {
            return None;
        }

        let next = unsafe { f(self.raw.add(self.count)) };
        self.count += 1;
        Some(next)
    }

    #[inline(always)]
    unsafe fn back<R>(&mut self, f: impl FnOnce(*mut T) -> R) -> Option<R> {
        if self.len == 0 || self.len == self.count {
            return None;
        }

        let count = self.len - self.count - 1;
        self.count += 1;
        unsafe { Some(f(self.raw.add(count))) }
    }
}

pub struct Iter<'a, T> {
    base: Base<T>,
    marker: PhantomData<fn() -> &'a T>,
}

impl<'a, T> Iter<'a, T> {
    pub(crate) fn new(source: &'a TypeErasedSparseSet) -> Self {
        Self {
            base: Base::new(source.raw.block.cast::<T>().as_ptr(), source.len),
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
    marker: PhantomData<fn() -> &'a mut T>,
}

impl<'a, T> IterMut<'a, T> {
    pub(crate) fn new(source: &'a mut TypeErasedSparseSet) -> Self {
        Self {
            base: Base::new(source.raw.block.cast::<T>().as_ptr(), source.len),
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
    use std::collections::HashMap;
    use super::*;
    use crate::entity::EntityId;

    #[derive(Debug, PartialEq)]
    struct Obj {
        name: String,
        age: u32,
    }

    impl Drop for Obj {
        fn drop(&mut self) {
            println!("dropped {} aged {}", self.name, self.age)
        }
    }

    impl Obj {
        fn new(name: &str, age: u32) -> Self {
            Self {
                name: name.to_string(),
                age,
            }
        }
    }

    #[test]
    fn get() -> Result<(), Error> {
        let entity = EntityId::new(0);
        let mut ss = TypeErasedSparseSet::new::<Obj>();
        ss.insert(entity, Obj::new("Balo", 69));

        let balo = ss.get::<EntityId, Obj>(entity);
        assert!(balo.is_some());

        Ok(())
    }

    #[test]
    fn swap_remove() -> Result<(), Error> {
        const NUM: usize = 5;
        let mut ss = TypeErasedSparseSet::new::<Obj>();
        let names = ["Balo", "Nunez", "Maguirre", "Bendtner", "Haryono"];

        for i in 0..NUM {
            let obj = Obj::new(names[i], i as _);
            ss.insert(EntityId::new(i as _), obj);
        }

        let last = EntityId::new(4);
        let to_remove = EntityId::new(1);

        let prev_index = ss.indexes.get_index(last);

        ss.swap_remove::<EntityId, Obj>(to_remove, last);
        let removed = ss.get::<EntityId, Obj>(to_remove);
        assert!(removed.is_none());

        let new_index = ss.indexes.get_index(last);
        assert_ne!(prev_index, new_index);

        println!("quitting");

        Ok(())
    }

    #[test]
    fn iter_speed() {
        struct NewObj {
            _name: String,
            age: usize,
            _addr: usize,
        }

        const NUM: usize = 1024 * 1024 * 4;

        let mut hm: HashMap<EntityId, NewObj> = HashMap::with_capacity(NUM);
        let now_1p = std::time::Instant::now();
        for i in 0..NUM { hm.insert(EntityId(i as _), NewObj { _name: i.to_string(), age: 1, _addr: i }); }
        let elapsed_1p = now_1p.elapsed();
        println!("HashMap<K, V>            : push time for {NUM} objects: {:?}", elapsed_1p);

        let now_3p = std::time::Instant::now();
        let mut any_hm: HashMap<EntityId, Box<dyn std::any::Any>> = HashMap::with_capacity(NUM);
        for i in 0..NUM { any_hm.insert(EntityId(i as _), Box::new(NewObj { _name: i.to_string(), age: 1, _addr: i })); }
        let elapsed_3p = now_3p.elapsed();
        println!("HashMap<K, Box<dyn Any>> : push time for {NUM} objects: {:?}", elapsed_3p);

        let now_2p = std::time::Instant::now();
        let mut te_ss = TypeErasedSparseSet::with_capacity::<NewObj>(NUM);
        for i in 0..NUM { te_ss.insert(EntityId(i as _), NewObj { _name: i.to_string(), age: 1, _addr: i }); }
        let elapsed_2p = now_2p.elapsed();
        println!("TypeErasedSparseSet      : push time for {NUM} objects: {:?}\n", elapsed_2p);

        let now_1i = std::time::Instant::now();
        let sum_1 = hm.iter().map(|(_, obj)| obj.age).sum::<usize>();
        let elapsed_1i = now_1i.elapsed();
        println!("HashMap<K, V>            : iter.map.sum time for {sum_1} objects: {:?}", elapsed_1i);

        let now_3i = std::time::Instant::now();
        let sum_3 = any_hm.iter().map(|(_, any)| any.downcast_ref::<NewObj>().unwrap().age).sum::<usize>();
        let elapsed_3i = now_3i.elapsed();
        println!("HashMap<K, Box<dyn Any>> : iter.map.sum time for {sum_3} objects: {:?}", elapsed_3i);

        let now_2i = std::time::Instant::now();
        let sum_2 = te_ss.iter::<NewObj>().map(|obj| obj.age).sum::<usize>();
        let elapsed_2i = now_2i.elapsed();
        println!("TypeErasedSparseSet      : iter.map.sum time for {sum_2} objects: {:?}", elapsed_2i);
    }
}
