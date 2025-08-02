use std::hash::{Hasher, BuildHasher};
use std::collections::HashMap;
use std::collections::hash_map::{
    Iter,
    IterMut,
    IntoIter,
    Drain,
};

pub struct U64Map<K, V>(pub(crate) HashMap<K, V, U64Hasher>);

impl<K, V> U64Map<K, V> {
    pub fn new() -> Self {
        Self(HashMap::with_hasher(U64Hasher(0)))
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity_and_hasher(capacity, U64Hasher(0)))
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        self.0.iter()
    }

    pub fn into_iter(self) -> IntoIter<K, V> {
        self.0.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        self.0.iter_mut()
    }

    pub fn drain(&mut self) -> Drain<'_, K, V> {
        self.0.drain()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl<K, V> U64Map<K, V>
where
    K: std::hash::Hash + Eq,
{
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.0.insert(k, v)
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.0.get(k)
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.0.get_mut(k)
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.0.remove(k)
    }

    pub fn contains_key(&self, k: &K) -> bool {
        self.0.contains_key(k)
    }
}

impl<K: Clone, V: Clone> Clone for U64Map<K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<K, V> Default for U64Map<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> std::ops::Index<&K> for U64Map<K, V>
where
    K: std::hash::Hash + Eq
{
    type Output = V;
    fn index(&self, index: &K) -> &Self::Output {
        &self.0[index]
    }
}

impl<K, V> std::fmt::Debug for U64Map<K, V>
where
    K: std::fmt::Debug,
    V: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.0.iter())
            .finish()
    }
}

#[derive(Clone)]
pub struct U64Hasher(u64);

impl Hasher for U64Hasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        panic!("use write_u64 instead")
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}

impl BuildHasher for U64Hasher {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        U64Hasher(0)
    }
}

#[cfg(test)]
mod hash_test {
    use super::*;
    use crate::index_map::IndexMap;
    use crate::entity;
    use crate::Entity;

    entity! { TestId }

    #[test]
    fn time_benchmark() {
        const REPEAT: usize = 1000000;
        let mut map = U64Map::new();
        let now = std::time::Instant::now();
        for i in 0..REPEAT {
            map.insert(TestId(i as u64, 0), i);
        }
        let time_map = now.elapsed();

        let mut hashmap = std::collections::HashMap::new();
        let now = std::time::Instant::now();
        for i in 0..REPEAT {
            hashmap.insert(TestId(i as u64, 0), i);
        }
        let time_std = now.elapsed();

        let mut indexmap = IndexMap::<TestId, usize>::new();
        let now = std::time::Instant::now();
        for i in 0..REPEAT {
            let _ = indexmap.insert(i);
        }
        let time_index = now.elapsed();

        eprintln!("INSERT: map: {time_map:?} | std: {time_std:?} | index_map: {time_index:?}");

        let now = std::time::Instant::now();
        for i in 0..REPEAT {
            let _ = map.get(&TestId(i as _, 0));
        }
        let time_get_map = now.elapsed();

        let now = std::time::Instant::now();
        for i in 0..REPEAT {
            let _ = hashmap.get(&TestId(i as _, 0));
        }
        let time_get_std = now.elapsed();

        let now = std::time::Instant::now();
        for i in 0..REPEAT {
            let _ = indexmap.get(&TestId(i as _, 0));
        }
        let time_get_indexmap = now.elapsed();

        eprintln!(
            "GET: map: {:?} | std: {:?} | index_map: {:?}",
            time_get_map,
            time_get_std,
            time_get_indexmap,
        );
    }
}
