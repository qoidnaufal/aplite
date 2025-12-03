use std::hash::{Hasher, BuildHasher};
use std::any::TypeId;
use std::collections::HashMap;

use crate::entity::EntityId;

pub type EntityIdMap<V> = HashMap<EntityId, V, NullHashBuilder>;

pub struct NullHash(u64);

impl Hasher for NullHash {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        panic!("Should never be called")
    }

    fn write_u8(&mut self, i: u8) { self.0 = i as _ }
    fn write_i8(&mut self, i: i8) { self.0 = i as _ }

    fn write_u16(&mut self, i: u16) { self.0 = i as _ }
    fn write_i16(&mut self, i: i16) { self.0 = i as _ }

    fn write_u32(&mut self, i: u32) { self.0 = i as _ }
    fn write_i32(&mut self, i: i32) { self.0 = i as _ }

    fn write_u64(&mut self, i: u64) { self.0 = i }
    fn write_i64(&mut self, i: i64) { self.0 = i as _ }

    fn write_u128(&mut self, i: u128) { self.0 = i as _ }
    fn write_i128(&mut self, i: i128) { self.0 = i as _ }

    fn write_usize(&mut self, i: usize) { self.0 = i as _ }
    fn write_isize(&mut self, i: isize) { self.0 = i as _ }
}

#[derive(Default)]
pub struct NullHashBuilder;

impl BuildHasher for NullHashBuilder {
    type Hasher = NullHash;

    fn build_hasher(&self) -> Self::Hasher {
        NullHash(0)
    }
}

pub struct TypeIdMap<V>(HashMap<TypeId, V, NullHashBuilder>);

impl<V> Default for TypeIdMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> TypeIdMap<V> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity_and_hasher(capacity, NullHashBuilder))
    }

    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn insert(&mut self, k: TypeId, v: V) -> Option<V> {
        self.0.insert(k, v)
    }

    pub fn remove(&mut self, k: &TypeId) -> Option<V> {
        self.0.remove(k)
    }

    pub fn retain<F: FnMut(&TypeId, &mut V) -> bool>(&mut self, f: F) {
        self.0.retain(f);
    }

    pub fn entry(&mut self, key: TypeId) -> std::collections::hash_map::Entry<'_, TypeId, V> {
        self.0.entry(key)
    }

    pub fn get(&self, k: &TypeId) -> Option<&V> {
        self.0.get(k)
    }

    pub fn get_mut(&mut self, k: &TypeId) -> Option<&mut V> {
        self.0.get_mut(k)
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, TypeId, V> {
        self.0.keys()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, TypeId, V> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<'_, TypeId, V> {
        self.0.iter_mut()
    }
}

impl<V: std::fmt::Debug> std::fmt::Debug for TypeIdMap<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter())
            .finish()
    }
}

impl<V> std::ops::Index<&TypeId> for TypeIdMap<V> {
    type Output = V;

    fn index(&self, index: &TypeId) -> &Self::Output {
        &self.0[index]
    }
}

impl<V> std::ops::IndexMut<&TypeId> for TypeIdMap<V> {
    fn index_mut(&mut self, index: &TypeId) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
