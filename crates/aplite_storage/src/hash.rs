use std::hash::{Hasher, BuildHasher};
use std::collections::HashMap;
use std::collections::hash_map::{Iter, IntoIter};

pub struct Map<K, V>(HashMap<K, V, U64Hasher>);

impl<K, V> Map<K, V> {
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

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<K, V> Map<K, V>
where
    K: std::hash::Hash + Eq,
{
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.0.insert(k, v)
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.0.get(k)
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.0.remove(k)
    }

    pub fn contains_key(&self, k: &K) -> bool {
        self.0.contains_key(k)
    }
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> std::ops::Index<&K> for Map<K, V>
where
    K: std::hash::Hash + Eq
{
    type Output = V;
    fn index(&self, index: &K) -> &Self::Output {
        &self.0[index]
    }
}

// impl<K, V> std::ops::IndexMut<&K> for Map<K, V>
// where
//     K: std::hash::Hash + Eq
// {
//     fn index_mut(&mut self, index: &K) -> &mut Self::Output {
//         &mut self.0[index]
//     }
// }

impl<K, V> std::fmt::Debug for Map<K, V>
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
    use std::hash::Hash;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct Id(u64, u32);

    impl Hash for Id {
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_u64(self.0);
        }
    }

    #[derive(Debug)]
    struct State {
        storage: Map<Id, String>,
    }

    impl State {
        fn new() -> Self {
            Self {
                storage: Map::new()
            }
        }
    }

    #[test]
    fn map() {
        let mut state = State::new();
        for i in 0..10 {
            let id = Id(i as u64, i as u32 * 2);
            state.storage.insert(id, i.to_string());
        }

        let id_zero = state.storage.get(&Id(0, 0));
        assert!(id_zero.is_some());

        eprintln!("{state:?}")
    }
}
