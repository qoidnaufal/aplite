use std::{iter::Enumerate, marker::PhantomData, slice::{Iter, IterMut}};

#[derive(Debug)]
pub struct MaxCapacityReached;

impl std::fmt::Display for MaxCapacityReached {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for MaxCapacityReached {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SlotContent<V> {
    Occupied(V),
    // contains index of next free slot
    Vacant(u32),
}

impl<V> SlotContent<V> {
    pub(crate) fn set_vacant(&mut self, idx: u32) -> Option<V> {
        let swap = std::mem::replace(self, Self::Vacant(idx));
        if let SlotContent::Occupied(v) = swap {
            Some(v)
        } else {
            None
        }
    }

    // pub(crate) fn occupy(&mut self, v: V) -> Option<V> {
    //     let old = std::mem::replace(self, Self::Occupied(v));
    //     if let SlotContent::Occupied(v) = old {
    //         Some(v)
    //     } else {
    //         None
    //     }
    // }
}

#[derive(Debug, Clone)]
pub struct Slot<V> {
    pub(crate) content: SlotContent<V>,
    pub(crate) version: u32,
}

impl<V> Slot<V> {
    pub(crate) fn vacant(pref_free_slot: u32) -> Self {
        Self {
            content: SlotContent::Vacant(pref_free_slot),
            version: 0,
        }
    }

    pub(crate) fn occupied(v: V) -> Self {
        Self {
            content: SlotContent::Occupied(v),
            version: 0,
        }
    }

    pub(crate) fn get_content(&self) -> Option<&V> {
        match &self.content {
            SlotContent::Occupied(v) => Some(v),
            SlotContent::Vacant(_) => None,
        }
    }

    pub(crate) fn get_content_mut(&mut self) -> Option<&mut V> {
        match &mut self.content {
            SlotContent::Occupied(v) => Some(v),
            SlotContent::Vacant(_) => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key<K: Sized> {
    pub(crate) idx: u32,
    pub(crate) version: u32,
    phantom: PhantomData<K>,
}

impl<K: Sized> std::fmt::Debug for Key<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = std::any::type_name::<K>()
            .split("::")
            .last()
            .unwrap();
        write!(f, "{} {{ version: {:03?} }}", type_name, self.version)
    }
}

impl<K: Sized> Key<K> {
    pub(crate) fn new(idx: u32, version: u32) -> Self {
        Self {
            idx,
            version,
            phantom: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct VecMap<K: Sized, V: Sized> {
    inner: Vec<Slot<V>>,
    free_slot: u32,
    count: u32,
    phantom: PhantomData<K>,
}

impl<K: Sized, V: Sized> Default for VecMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Sized, V: Sized> std::ops::Index<Key<K>> for VecMap<K, V> {
    type Output = V;
    fn index(&self, key: Key<K>) -> &Self::Output {
        let slot = &self.inner[key.idx as usize];
        if key.version == slot.version {
            slot.get_content().unwrap()
        } else {
            unreachable!()
        }
    }
}

impl<K: Sized, V: Sized> std::ops::IndexMut<Key<K>> for VecMap<K, V> {
    fn index_mut(&mut self, key: Key<K>) -> &mut Self::Output {
        let slot = &mut self.inner[key.idx as usize];
        if key.version == slot.version {
            slot.get_content_mut().unwrap()
        } else {
            unreachable!()
        }
    }
}

impl<K: Sized, V: Sized> VecMap<K, V> {
    pub fn new() -> Self {
        Self::new_with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::new_with_capacity(capacity)
    }

    fn new_with_capacity(capacity: usize) -> Self {
        let mut inner = Vec::with_capacity(capacity + 1);
        let slot = Slot::vacant(1);
        inner.push(slot);
        Self {
            inner,
            free_slot: 0,
            count: 0,
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize { self.count as usize }

    pub fn is_empty(&self) -> bool { self.count == 0 }

    pub fn insert(&mut self, v: V) -> Key<K> {
        self.insert_with_key(|_| Ok(v))
    }

    pub fn insert_with_key<F>(&mut self, f: F) -> Key<K>
    where
        F: FnOnce(&Key<K>) -> Result<V, MaxCapacityReached>
    {
        self.try_insert_with_key(f).unwrap()
    }

    pub fn try_insert_with_key<F>(&mut self, f: F) -> Result<Key<K>, MaxCapacityReached>
    where
        F: FnOnce(&Key<K>) -> Result<V, MaxCapacityReached>,
    {
        if self.count + 1 == u32::MAX { return Err(MaxCapacityReached) }

        // check vacant spot first
        if let Some(slot) = self.inner.get_mut(self.free_slot as usize) {
            let next = match slot.content {
                SlotContent::Vacant(idx) => idx,
                SlotContent::Occupied(_) => self.free_slot,
            };
            let key = Key::new(self.free_slot, slot.version);
            let v = f(&key)?;
            slot.content = SlotContent::Occupied(v);

            self.count += 1;
            self.free_slot = next;

            return Ok(key);
        }

        let key = Key::new(self.free_slot, 0);
        let v = f(&key)?;
        self.inner.push(Slot::occupied(v));
        self.count += 1;
        self.free_slot += 1;

        Ok(key)
    }

    pub fn get(&self, key: &Key<K>) -> Option<&V> {
        let slot = &self.inner[key.idx as usize];
        if slot.version == key.version {
            slot.get_content()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &Key<K>) -> Option<&mut V> {
        let slot = &mut self.inner[key.idx as usize];
        if slot.version == key.version {
            slot.get_content_mut()
        } else {
            None
        }
    }

    pub fn contains(&self, key: &Key<K>) -> bool {
        let slot = &self.inner[key.idx as usize];
        slot.version == key.version
    }

    pub fn remove(&mut self, key: &Key<K>) -> Option<V> {
        if self.contains(key) {
            self.remove_with_index(key.idx as usize)
        } else {
            None
        }
    }

    pub fn remove_with_index(&mut self, index: usize) -> Option<V> {
        if let Some(slot) = self.inner.get_mut(index) {
            let v = slot.content.set_vacant(self.free_slot);
            slot.version += 1;
            self.free_slot = index as u32;
            self.count -= 1;

            v
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.inner.push(Slot::vacant(1));
        self.count = 0;
        self.free_slot = 0;
    }

    pub fn iter(&self) -> VecMapIter<'_, K, V> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> VecMapIterMut<'_, K, V> {
        self.into_iter()
    }
}

impl<K: Sized, V: std::fmt::Debug> std::fmt::Debug for VecMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.count == 0 {
            write!(f, "[]")
        } else {
            let to_write = self
                .inner
                .iter()
                .filter_map(|slot| {
                    match &slot.content {
                        SlotContent::Occupied(v) => Some(v),
                        SlotContent::Vacant(_) => None,
                    }
                })
                .collect::<Vec<_>>();
            write!(f, "{to_write:?}")
        }
    }
}

pub struct VecMapIter<'a, K, V> {
    pub(crate) inner: Enumerate<Iter<'a, Slot<V>>>,
    pub(crate) counter: usize,
    pub(crate) phantom: PhantomData<K>,
}

impl<'a, K, V> Iterator for VecMapIter<'a, K, V> {
    type Item = (Key<K>, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .and_then(|(idx, slot)| {
                slot.get_content()
                    .map(|v| {
                        self.counter -= 1;
                        (Key::new(idx as u32, slot.version), v)
                    })
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.counter, Some(self.counter))
    }
}

impl<'a, K, V> IntoIterator for &'a VecMap<K, V> {
    type Item = (Key<K>, &'a V);
    type IntoIter = VecMapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        VecMapIter {
            inner: self.inner.iter().enumerate(),
            counter: self.len(),
            phantom: PhantomData
        }
    }
}

pub struct VecMapIterMut<'a, K, V> {
    pub(crate) inner: Enumerate<IterMut<'a, Slot<V>>>,
    pub(crate) counter: usize,
    pub(crate) phantom: PhantomData<K>,
}

impl<'a, K, V> Iterator for VecMapIterMut<'a, K, V> {
    type Item = (Key<K>, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .and_then(|(idx, slot)| {
                let version = slot.version;
                slot.get_content_mut()
                    .map(|v| {
                        self.counter -= 1;
                        (Key::new(idx as u32, version), v)
                    })
            })
    }
}

impl<'a, K, V> IntoIterator for &'a mut VecMap<K, V> {
    type Item = (Key<K>, &'a mut V);
    type IntoIter = VecMapIterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        VecMapIterMut {
            counter: self.len(),
            inner: self.inner.iter_mut().enumerate(),
            phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod vecmap {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct MyKey(u32);

    #[test]
    fn insert() {
        let mut storage: VecMap<MyKey, u8> = VecMap::new();
        let key = storage.insert(69);
        assert_eq!(key.idx, 0)
    }

    #[test]
    fn get() {
        let mut storage: VecMap<MyKey, u8> = VecMap::new();
        let key = storage.insert(69);
        let res = storage.get(&key).unwrap();
        assert_eq!(*res, 69)
    }

    #[test]
    fn remove() {
        let mut storage: VecMap<MyKey, u8> = VecMap::new();
        let mut keys = vec![];
        for i in 0..10u8 {
            let key = storage.insert(i);
            keys.push(key);
        }
        assert!(storage.inner.last().is_some_and(|slot| *slot.get_content().unwrap() == 9));

        let key = &keys[3];
        let removed = storage.remove(key);
        assert_eq!(removed, Some(3));

        let after_remove = storage.get(key);
        assert_eq!(after_remove, None);

        let new_key = storage.insert(100);
        assert_ne!(&new_key, key);

        let none_get = storage.get(key);
        assert_eq!(none_get, None);

        let none_remove = storage.remove(key);
        assert_eq!(none_remove, None);
    }

    #[test]
    fn double_remove() {
        let mut storage: VecMap<MyKey, u8> = VecMap::new();
        let mut keys = vec![];
        for i in 0..10u8 {
            let key = storage.insert(i);
            keys.push(key);
        }

        for key in &keys[1..3] {
            let _ = storage.remove(key);
        }

        assert_eq!(storage.free_slot, 2);
        assert_eq!(storage.inner[1].content, SlotContent::Vacant(10));
        assert_eq!(storage.inner[2].content, SlotContent::Vacant(1));

        let _ = storage.insert(69);
        assert_eq!(storage.free_slot, 1);

        let _ = storage.insert(100);
        assert_eq!(storage.free_slot, 10);
    }
}
