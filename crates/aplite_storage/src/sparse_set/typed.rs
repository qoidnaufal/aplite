use std::marker::PhantomData;

use crate::ArenaPtr;
use crate::buffer::Error;
use crate::sparse_set::type_erased::{Iter, IterMut};
use crate::sparse_set::SparsetKey;
use crate::sparse_set::type_erased::TypeErasedSparseSet;

pub struct SparseSet<K, V> {
    inner: TypeErasedSparseSet,
    marker: PhantomData<(K, V)>
}

impl<K: SparsetKey, V> Default for SparseSet<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: SparsetKey, V: std::fmt::Debug> std::fmt::Debug for SparseSet<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(
            self.inner.indexes
                .iter_data_index()
                .zip(self.iter())
        )
        .finish()
    }
}

impl<K: SparsetKey, V> SparseSet<K, V> {
    pub fn new() -> Self {
        Self {
            inner: TypeErasedSparseSet::new::<V>(),
            marker: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: TypeErasedSparseSet::with_capacity::<V>(capacity),
            marker: PhantomData,
        }
    }

    pub unsafe fn get_raw(&mut self, key: K) -> Option<*mut V> {
        unsafe {
            self.inner
                .get_raw::<K>(key)
                .map(|raw| raw.cast())
        }
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.inner.get::<K, V>(key)
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.inner.get_mut::<K, V>(key)
    }

    pub unsafe fn insert_unchecked(&mut self, key: K, value: V) -> ArenaPtr<V> {
        unsafe { self.inner.insert_unchecked(key, value) }
    }

    pub fn insert_within_capacity(&mut self, key: K, value: V) -> Result<ArenaPtr<V>, Error> {
        self.inner.insert_within_capacity(key, value)
    }

    /// Inserting or replacing the value
    pub fn insert(&mut self, key: K, value: V) -> ArenaPtr<V> {
        self.inner.insert(key, value)
    }

    pub fn replace(&mut self, key: K, value: V) -> Option<ArenaPtr<V>> {
        self.inner.replace::<K, V>(key, value)
    }

    /// The contiguousness of the data is guaranteed after removal via [`Vec::swap_remove`],
    /// but the order of the data is is not.
    pub fn swap_remove(&mut self, key: K, last_key_to_swap: K) -> Option<V> {
        self.inner.swap_remove::<K, V>(key, last_key_to_swap)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// The length of the data
    pub fn len(&self) -> usize {
        self.inner.len
    }

    /// Check if the data is empty or not
    pub fn is_empty(&self) -> bool {
        self.inner.len == 0
    }

    pub fn contains_key(&self, key: K) -> bool {
        self.inner.contains_key(key)
    }

    pub fn iter(&self) -> Iter<'_, V> {
        Iter::new(&self.inner)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, V> {
        IterMut::new(&mut self.inner)
    }

    pub fn iter_data_index(&self) -> impl Iterator<Item = usize> {
        self.inner.indexes.iter_data_index()
    }
}

/*
#########################################################
#
# Test
#
#########################################################
*/

#[cfg(test)]
mod typed_sparse_set {
    use super::*;
    use crate::entity::EntityId;

    struct Obj {
        name: String,
        age: u32,
    }

    impl std::fmt::Debug for Obj {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Obj")
                .field("name", &self.name)
                .field("age", &self.age)
                .finish()
        }
    }

    impl From<u32> for Obj {
        fn from(value: u32) -> Self {
            Self {
                name: value.to_string(),
                age: value,
            }
        }
    }

    #[test]
    fn get_test() {
        let mut tss = SparseSet::<EntityId, Obj>::with_capacity(5);
        for i in 0..5 {
            let id = EntityId::new(i);
            tss.insert(id, i.into());
        }

        let res = tss.get(EntityId::new(1));
        assert!(res.is_some());
        println!("{:?}", res.unwrap())
    }
}
