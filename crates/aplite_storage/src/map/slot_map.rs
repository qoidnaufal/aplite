use core::slice::{Iter as SliceIter, IterMut as SliceIterMut};
use core::iter::{FilterMap, Enumerate};

use super::slot::Slot;
use super::id::SlotId;

/*
#########################################################
#
# SlotMap
#
#########################################################
*/

pub struct SlotMap<T> {
    pub(crate) inner: Vec<Slot<T>>,
    next: u32,
    count: u32,
}

impl<T: Clone> Clone for SlotMap<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            next: self.next,
            count: self.count,
        }
    }
}

impl<T> Default for SlotMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SlotMap<T> {
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            next: 0,
            count: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            next: 0,
            count: 0,
        }
    }

    /// Panics if there are already u32::MAX of stored elements. Use [`try_insert`](IndexMap::try_insert) if you want to handle the error manually.
    pub fn insert(&mut self, data: T) -> SlotId {
        self.try_insert(data).unwrap()
    }

    pub fn try_insert(&mut self, data: T) -> Result<SlotId, Error<T>> {
        if self.count == u32::MAX { return Err(Error::ReachedMaxCapacity(data)) }

        match self.inner.get_mut(self.next as usize) {
            // after removal
            Some(slot) => unsafe {
                let next_id = slot.content.next_id;
                let id = SlotId::new(self.next, slot.version);

                slot.occupy(data);
                self.next = next_id;
                self.count += 1;

                Ok(id)
            }
            None => {
                let id = SlotId::new(self.next, 0);
                self.inner.push(Slot::new(data));
                self.next += 1;
                self.count += 1;
                Ok(id)
            },
        }
    }

    pub fn remove(&mut self, index: SlotId) -> Option<T> {
        self.inner
            .get_mut(index.index())
            .and_then(|slot| {
                slot.validate_occupied(index.version).then(|| {
                    let removed = slot.set_vacant(self.next);
                    self.next = index.index;
                    self.count -= 1;
                    removed
                })
            })
    }

    pub fn replace(&mut self, index: &SlotId, data: T) -> Result<T, Error<T>> {
        match self.inner.get_mut(index.index()) {
            Some(slot) => {
                let Some(prev) = slot.get_validated_mut(index.version) else {
                    return Err(Error::InvalidSlot(data))
                };
                Ok(core::mem::replace(prev, data))
            },
            None => Err(Error::InvalidIndex(data)),
        }
    }

    pub fn get(&self, index: &SlotId) -> Option<&T> {
        self.inner
            .get(index.index())
            .and_then(|slot| slot.get_validated(index.version))
    }

    pub fn get_mut(&mut self, index: &SlotId) -> Option<&mut T> {
        self.inner
            .get_mut(index.index())
            .and_then(|slot| slot.get_validated_mut(index.version))
    }

    pub fn contains(&self, index: &SlotId) -> bool {
        self.inner
            .get(index.index())
            .is_some_and(|slot| slot.validate_occupied(index.version))
    }

    pub fn len(&self) -> usize { self.count as usize }

    pub fn is_empty(&self) -> bool { self.count == 0 }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.next = 0;
        self.count = 0;
    }

    pub fn iter(&self) -> Iter<'_, T> {
        let inner = self
            .inner
            .iter()
            .enumerate()
            .filter_map(filter_ref as FnFilterRef<T>);

        Iter { inner }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        let inner = self
            .inner
            .iter_mut()
            .enumerate()
            .filter_map(filter_mut as FnFilterMut<T>);

        IterMut { inner }
    }
}

impl<T> std::fmt::Debug for SlotMap<T>
where
    T: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.iter())
            .finish()
    }
}

impl<T> std::ops::Index<SlotId> for SlotMap<T> {
    type Output = T;

    fn index(&self, index: SlotId) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

impl<T> std::ops::IndexMut<SlotId> for SlotMap<T> {
    fn index_mut(&mut self, index: SlotId) -> &mut Self::Output {
        self.get_mut(&index).unwrap()
    }
}

/// It's important to return the data here, in case non-copy data is being used and data is needed in error handling
pub enum Error<T> {
    ReachedMaxCapacity(T),
    InvalidIndex(T),
    InvalidSlot(T),
}

impl<T> std::fmt::Debug for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Error::ReachedMaxCapacity(_) => "ReachedMaxCapacity",
            Error::InvalidIndex(_) => "InvalidIndex",
            Error::InvalidSlot(_) => "InvalidSlot",
        };

        write!(f, "{msg}")
    }
}

impl<T> std::fmt::Display for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl<T> std::error::Error for Error<T> {}

/*
#########################################################
#
# Iter<'_, T>
#
#########################################################
*/

fn filter_ref<T>((i, slot): (usize, &Slot<T>)) -> Option<(SlotId, Option<&T>)> {
    slot.get().map(|data| (SlotId::new(i as _, slot.version), Some(data)))
}

type FnFilterRef<T> = fn((usize, &Slot<T>)) -> Option<(SlotId, Option<&T>)>;

pub struct Iter<'a, T> {
    #[allow(clippy::type_complexity)]
    inner: FilterMap<Enumerate<SliceIter<'a, Slot<T>>>, FnFilterRef<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (SlotId, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(id, val)| (id, val.unwrap()))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(|(id, val)| (id, val.unwrap()))
    }
}

/*
#########################################################
#
# IterMut<'_, T>
#
#########################################################
*/

fn filter_mut<T>((i, slot): (usize, &mut Slot<T>)) -> Option<(SlotId, Option<&mut T>)> {
    let version = slot.version;
    slot.get_mut().map(|data| (SlotId::new(i as _, version), Some(data)))
}

type FnFilterMut<T> = fn((usize, &mut Slot<T>)) -> Option<(SlotId, Option<&mut T>)>;

pub struct IterMut<'a, T> {
    #[allow(clippy::type_complexity)]
    inner: FilterMap<Enumerate<SliceIterMut<'a, Slot<T>>>, FnFilterMut<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (SlotId, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(id, val)| (id, val.unwrap()))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(|(id, val)| (id, val.unwrap()))
    }
}

#[cfg(test)]
mod slot_map_test {
    use super::*;

    #[test]
    fn insert_get() {
        let mut storage = SlotMap::new();
        let mut created_ids = Vec::with_capacity(10);

        for i in 0..10 {
            let id = storage.insert(i);
            created_ids.push(id);
        }

        // println!("{:?}", storage.get(&SlotId::new(0, 0)));
        assert!(created_ids.iter().all(|id| storage.get(id).is_some()));
        assert_eq!(storage.len(), created_ids.len());
    }

    #[test]
    fn remove_index() {
        let mut storage = SlotMap::new();
        let mut created_ids = Vec::with_capacity(10);

        for _ in 0..10 {
            let id = storage.insert(());
            created_ids.push(id);
        }

        created_ids.iter().for_each(|id| storage.remove(*id).unwrap());

        let mut new_ids = Vec::with_capacity(10);

        for _ in 0..10 {
            let new_id = storage.insert(());
            new_ids.push(new_id);
        }

        new_ids.sort_by_key(|id| id.index);

        assert!(created_ids.iter().zip(new_ids.iter()).all(|(old, new)| old.index == new.index));
        assert!(created_ids.iter().zip(new_ids.iter()).all(|(old, new)| old.version != new.version));
    }

    #[test]
    fn iter() {
        let mut storage = SlotMap::with_capacity(10);
        let mut created_ids = vec![];

        for i in 0..10 {
            let id = storage.insert(i);
            created_ids.push(id);
        }

        assert_eq!(storage.len(), created_ids.len());

        for i in 0..3 {
            storage.remove(created_ids[i * 3]);
        }

        let remaining = storage.iter().count();
        assert_eq!(remaining, storage.len());
    }
}
