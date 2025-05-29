use std::{iter::Enumerate, marker::PhantomData, slice::{Iter, IterMut}};

use crate::vecmap::Key;

#[derive(Debug, Clone, PartialEq, Eq)]
enum DerivedSlotContent<V> {
    Occupied(V),
    Vacant,
}

impl<V> DerivedSlotContent<V> {
    pub(crate) fn get(&self) -> Option<&V> {
        match self {
            DerivedSlotContent::Occupied(v) => Some(v),
            DerivedSlotContent::Vacant => None,
        }
    }

    pub(crate) fn get_mut(&mut self) -> Option<&mut V> {
        match self {
            DerivedSlotContent::Occupied(v) => Some(v),
            DerivedSlotContent::Vacant => None,
        }
    }

    pub(crate) fn set_vacant(&mut self) -> Option<V> {
        let swap = std::mem::replace(self, Self::Vacant);
        if let DerivedSlotContent::Occupied(v) = swap {
            Some(v)
        } else {
            None
        }
    }

    pub(crate) fn occupy(&mut self, v: V) -> Option<V> {
        let old = std::mem::replace(self, Self::Occupied(v));
        if let DerivedSlotContent::Occupied(v) = old {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct DerivedSlot<V> {
    pub(crate) content: DerivedSlotContent<V>,
    pub(crate) version: u32,
}

impl<V> DerivedSlot<V> {
    pub(crate) fn vacant() -> Self {
        Self {
            content: DerivedSlotContent::Vacant,
            version: 0,
        }
    }
}

#[derive(Clone)]
pub struct DerivedMap<K: Sized, V: Sized> {
    inner: Vec<DerivedSlot<V>>,
    count: u32,
    phantom: PhantomData<K>,
}

impl<K: Sized, V: Sized> Default for DerivedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Sized, V: Sized> DerivedMap<K, V> {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            count: 0,
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize { self.count as usize }

    pub fn is_empty(&self) -> bool { self.count == 0 }

    pub fn insert(&mut self, k: &Key<K>, v: V) -> Option<V> {
        if k.idx >= self.count {
            self.inner.extend((self.count..=k.idx).map(|_| DerivedSlot::vacant()));
        }
        self.count += 1;

        let slot = &mut self.inner[k.idx as usize];
        slot.version = k.version;
        slot.content.occupy(v)
    }

    pub fn entry<'a>(&'a mut self, k: &'a Key<K>) -> DerivedEntry<'a, K, V> {
        if self.contains(k) {
            return DerivedEntry::Occupied(self.get_mut(k).unwrap());
        }

        DerivedEntry::Vacant { key: k, inner: self }
    }

    pub fn get(&self, k: &Key<K>) -> Option<&V> {
        self.inner
            .get(k.idx as usize)
            .and_then(|slot| {
                if k.version == slot.version {
                    slot.content.get()
                } else {
                    None
                }
            })
    }

    pub fn get_mut(&mut self, k: &Key<K>) -> Option<&mut V> {
        self.inner
            .get_mut(k.idx as usize)
            .and_then(|slot| {
                if k.version == slot.version {
                    slot.content.get_mut()
                } else {
                    None
                }
            })
    }

    pub fn contains(&self, k: &Key<K>) -> bool {
        self.inner
            .get(k.idx as usize)
            .is_some_and(|s| {
                let occupied = s.content.get().is_some();
                let version = k.version == s.version;
                occupied & version
            })
    }

    pub fn remove(&mut self, k: &Key<K>) -> Option<V> {
        self.inner
            .get_mut(k.idx as usize)
            .and_then(|slot| {
                if k.version == slot.version {
                    slot.content.set_vacant()
                } else {
                    None
                }
            })
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.count = 0;
    }

    pub fn iter(&self) -> DerivedIter<'_, K, V> {
        self.into_iter()
    }
}

impl<K: Sized, V: std::fmt::Debug> std::fmt::Debug for DerivedMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "[]")
        } else {
            let to_write = self.inner
                .iter()
                .filter_map(|slot| {
                    match &slot.content {
                        DerivedSlotContent::Occupied(v) => Some(v),
                        DerivedSlotContent::Vacant => None,
                    }
                })
                .collect::<Vec<_>>();
            write!(f, "{to_write:?}")
        }
    }
}

pub struct DerivedIter<'a, K, V> {
    inner: Enumerate<Iter<'a, DerivedSlot<V>>>,
    counter: usize,
    phantom: PhantomData<K>,
}

impl<'a, K, V> Iterator for DerivedIter<'a, K, V> {
    type Item = (Key<K>, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .and_then(|(idx, slot)| {
                slot.content
                    .get()
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

impl<'a, K, V> IntoIterator for &'a DerivedMap<K, V> {
    type Item = (Key<K>, &'a V);
    type IntoIter = DerivedIter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        DerivedIter {
            inner: self.inner.iter().enumerate(),
            counter: self.len(),
            phantom: PhantomData,
        }
    }
}

pub struct DerivedIterMut<'a, K, V> {
    inner: Enumerate<IterMut<'a, DerivedSlot<V>>>,
    counter: usize,
    phantom: PhantomData<K>,
}

impl<'a, K, V> Iterator for DerivedIterMut<'a, K, V> {
    type Item = (Key<K>, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .and_then(|(idx, slot)| {
                slot.content
                    .get_mut()
                    .map(|v| {
                        self.counter -= 1;
                        (Key::new(idx as u32, slot.version), v)
                    })
            })
    }
}

impl<'a, K, V> IntoIterator for &'a mut DerivedMap<K, V> {
    type Item = (Key<K>, &'a mut V);
    type IntoIter = DerivedIterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        DerivedIterMut {
            counter: self.len(),
            inner: self.inner.iter_mut().enumerate(),
            phantom: PhantomData,
        }
    }
}

pub enum DerivedEntry<'a, K, V> {
    Vacant {
        key: &'a Key<K>,
        inner: &'a mut DerivedMap<K, V>,
    },
    Occupied(&'a mut V),
}

impl<'a, K, V: Default> DerivedEntry<'a, K, V> {
    pub fn or_default(self) -> &'a mut V {
        match self {
            DerivedEntry::Vacant { key, inner } => {
                inner.inner.extend((inner.count..=key.idx).map(|_| DerivedSlot::vacant()));
                let slot = &mut inner.inner[key.idx as usize];
                slot.content = DerivedSlotContent::Occupied(V::default());
                inner.count += 1;

                slot.content.get_mut().unwrap()
            },
            DerivedEntry::Occupied(v) => v,
        }
    }
}

#[cfg(test)]
mod dmap_test {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct MyKey;

    #[test]
    fn insert() {
        let mut dmap: DerivedMap<MyKey, u32> = DerivedMap::new();
        let mut keys = vec![];
        let mut olds = vec![];

        for i in 0..10u32 {
            let key = Key::<MyKey>::new(9 - i, i * 2);
            let old = dmap.insert(&key, i);
            keys.push(key);
            olds.push(old);
        }

        let key = &keys[0];
        assert_eq!(dmap.get(key), Some(&0));
        assert_eq!(key.idx, 9);
        assert!(olds.iter().all(|v| v.is_none()));
    }

    #[test]
    fn entry() {
        let mut dmap: DerivedMap<MyKey, Vec<u32>> = DerivedMap::new();
        let mut keys = vec![];
        for i in 0..10u32 {
            let key = Key::<MyKey>::new(i & 1, 0);
            dmap.entry(&key).or_default().push(i);
            if !keys.contains(&key) { keys.push(key) }
        }

        let even_k = &keys[0];
        let even = dmap.get(even_k).unwrap().iter().all(|num| num % 2 == 0);
        assert!(even);
    }
}
