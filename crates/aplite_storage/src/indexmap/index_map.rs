use crate::entity::EntityId;
use crate::iterator::{IndexMapIter, IndexMapIterMut};
use super::slot::{Slot, Content};

/// Arena style non-contiguous key-value data storage. [`IndexMap`] facilitates the creation of [`Entity`].
pub struct IndexMap<T> {
    pub(crate) inner: Vec<Slot<T>>,
    next: u32,
    count: u32,
}

impl<T> Default for IndexMap<T> {
    fn default() -> Self {
        Self::new_with_capacity(0)
    }
}

impl<T: PartialEq> IndexMap<T> {
    pub fn insert_no_duplicate(&mut self, data: T) -> EntityId {
        self.try_insert_no_duplicate(data).unwrap()
    }

    #[inline(always)]
    pub fn try_insert_no_duplicate(&mut self, data: T) -> Result<EntityId, IndexMapError> {
        if let Some((idx, slot)) = self.inner
            .iter()
            .enumerate()
            .find(|(_, slot)| {
                slot.get_content()
                    .is_some_and(|content| content == &data)
            }) {
            Ok(EntityId::new(idx as u32, slot.version))
        } else {
            self.try_insert(data)
        }
    }
}

impl<T> IndexMap<T> {
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::new_with_capacity(capacity)
    }

    /// This would ensure best performance on [`insert`](Self::insert),
    /// but kinda wasteful if you don't really use all the capacity.
    #[inline(always)]
    pub fn with_max_capacity() -> Self {
        Self::new_with_capacity(EntityId::INDEX_MASK as usize)
    }

    #[inline(always)]
    fn new_with_capacity(capacity: usize) -> Self {
        let mut inner = Vec::with_capacity(capacity + 1);
        inner.push(Slot {
            version: 0,
            content: Content::Vacant(1),
        });
        Self {
            inner,
            next: 0,
            count: 0,
        }
    }

    /// Panic if the generated [`Entity`] has reached [`u64::MAX`], or there's an internal error.
    /// Use [`try_insert`](IndexMap::try_insert) if you want to handle the error manually
    #[inline(always)]
    pub fn insert(&mut self, data: T) -> EntityId {
        self.try_insert(data).unwrap()
    }

    #[inline(always)]
    pub fn try_insert(&mut self, data: T) -> Result<EntityId, IndexMapError> {
        if self.count > EntityId::INDEX_MASK { return Err(IndexMapError::ReachedMaxId) }

        match self.inner.get_mut(self.next as usize) {
            // first time or after removal
            Some(slot) => match slot.content {
                Content::Occupied(_) => Err(IndexMapError::InternalCollision),
                Content::Vacant(idx) => {
                    let entity = EntityId::new(self.next, slot.version);
                    self.next = idx;
                    self.count += 1;
                    slot.content = Content::Occupied(data);

                    Ok(entity)
                },
            }
            None => {
                let entity = EntityId::new(self.next, 0);
                self.inner.push(Slot {
                    version: 0,
                    content: Content::Occupied(data),
                });
                self.count += 1;
                self.next += 1;

                Ok(entity)
            },
        }
    }

    /// Return None if the index is invalid.
    /// Use [`try_replace`](IndexMap::try_replace()) if you want to handle the error manually
    #[inline(always)]
    pub fn replace(&mut self, id: &EntityId, data: T) -> Option<T> {
        self.try_replace(id, data).ok()
    }

    #[inline(always)]
    pub fn try_replace(&mut self, id: &EntityId, data: T) -> Result<T, IndexMapError> {
        match self.inner.get_mut(id.index()) {
            Some(slot) if id.version() == slot.version => {
                slot.get_content_mut()
                    .ok_or(IndexMapError::InvalidSlot)
                    .map(|prev| std::mem::replace(prev, data))
            },
            _ => Err(IndexMapError::InvalidId),
        }
    }

    #[inline(always)]
    pub fn get(&self, id: &EntityId) -> Option<&T> {
        self.inner
            .get(id.index())
            .and_then(|slot| {
                if slot.version == id.version() {
                    slot.get_content()
                } else {
                    None
                }
            })
    }

    #[inline(always)]
    pub fn get_mut(&mut self, id: &EntityId) -> Option<&mut T> {
        self.inner
            .get_mut(id.index())
            .and_then(|slot| {
                if slot.version == id.version() {
                    slot.get_content_mut()
                } else {
                    None
                }
            })
    }

    #[inline(always)]
    pub fn remove(&mut self, id: &EntityId) -> Option<T> {
        if let Some(slot) = self.inner.get_mut(id.index())
            && slot.version == id.version()
        {
            let ret = std::mem::replace(&mut slot.content, Content::Vacant(self.next));
            slot.version += 1;
            self.next = id.index() as u32;
            self.count -= 1;

            if self.inner.len() as u32 - self.count > 4 {
                self.inner.shrink_to_fit();
            }

            match ret {
                Content::Occupied(data) => Some(data),
                Content::Vacant(_) => None,
            }
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn contains(&self, id: &EntityId) -> bool {
        self.inner
            .get(id.index())
            .is_some_and(|slot| slot.version == id.version())
    }

    #[inline(always)]
    pub fn len(&self) -> usize { self.count as usize }

    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.count == 0 }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.inner.clear();
        self.inner.push(Slot {
            version: 0,
            content: Content::Vacant(1),
        });
        self.next = 0;
        self.count = 0;
    }

    #[inline(always)]
    pub fn iter(&self) -> IndexMapIter<'_, T> { self.into_iter() }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> IndexMapIterMut<'_, T> { self.into_iter() }
}

impl<T> std::fmt::Debug for IndexMap<T>
where
    T: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.iter())
            .finish()
    }
}

impl<T> std::ops::Index<EntityId> for IndexMap<T> {
    type Output = T;
    fn index(&self, index: EntityId) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

impl<T: Clone> Clone for IndexMap<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            next: self.next,
            count: self.count,
        }
    }
}

#[derive(Debug)]
pub enum IndexMapError {
    ReachedMaxId,
    InternalCollision,
    InvalidId,
    InvalidSlot,
}

impl std::fmt::Display for IndexMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for IndexMapError {}

#[cfg(test)]
mod index_test {
    use super::*;

    #[test]
    fn insert_get() {
        let mut storage = IndexMap::<()>::with_capacity(10);
        let mut created_ids = Vec::with_capacity(10);

        for _ in 0..10 {
            let id = storage.insert(());
            created_ids.push(id);
        }

        assert!(created_ids.iter().all(|id| storage.get(id).is_some()));
        assert_eq!(storage.len(), created_ids.len());
    }

    #[test]
    fn remove_index() {
        let mut storage = IndexMap::<()>::with_capacity(10);
        let mut created_ids = Vec::with_capacity(10);

        for _ in 0..10 {
            let id = storage.insert(());
            created_ids.push(id);
        }

        created_ids.iter().for_each(|id| storage.remove(id).unwrap());

        let mut new_ids = Vec::with_capacity(10);
        for _ in 0..10 {
            let new_id = storage.insert(());
            new_ids.push(new_id);
        }

        assert_ne!(created_ids, new_ids);
    }

    #[test]
    fn no_duplicate() {
        let mut storage = IndexMap::<String>::with_capacity(10);
        let mut created_ids = vec![];

        for i in 0..10 {
            let data = if i > 0 && i % 3 == 0 {
                "Double".to_string()
            } else {
                i.to_string()
            };
            let id = storage.insert_no_duplicate(data);
            if !created_ids.contains(&id) {
                created_ids.push(id);
            }
        }

        assert_eq!(storage.len(), created_ids.len());
    }
}
