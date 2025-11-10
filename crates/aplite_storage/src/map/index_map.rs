use crate::entity::Entity;
use crate::iterator::{IndexMapIter, IndexMapIterMut};
use super::slot::Slot;

/// Arena style non-contiguous key-value data storage. Facilitates the creation of [`Entity`].
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

impl<T> IndexMap<T> {
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::new_with_capacity(capacity)
    }

    /// This would ensure best performance on [`insert`](Self::insert),
    /// but kinda wasteful if you don't really use all the capacity.
    #[inline(always)]
    pub fn with_max_capacity() -> Self {
        Self::new_with_capacity(u32::MAX as usize)
    }

    #[inline(always)]
    fn new_with_capacity(capacity: usize) -> Self {
        let mut inner = Vec::with_capacity(capacity + 1);
        inner.push(Slot::new());
        Self {
            inner,
            next: 0,
            count: 0,
        }
    }

    /// Panic if the generated [`Entity`] has reached [`u64::MAX`], or there's an internal error.
    /// Use [`try_insert`](IndexMap::try_insert) if you want to handle the error manually
    #[inline(always)]
    pub fn insert(&mut self, data: T) -> Entity {
        self.try_insert(data).unwrap()
    }

    #[inline(always)]
    pub fn try_insert(&mut self, data: T) -> Result<Entity, IndexMapError> {
        if self.count == u32::MAX { return Err(IndexMapError::ReachedMaxId) }

        match self.inner.get_mut(self.next as usize) {
            // first time or after removal
            Some(slot) => slot.occupy(data)
                .map(|next| {
                    let id = Entity::new(self.next, slot.version);
                    self.next = next;
                    self.count += 1;
                    id
                })
                .ok_or(IndexMapError::InvalidSlot),
            None => {
                let entity = Entity::new(self.next, 0);
                self.inner.push(Slot::with_data(data));
                self.count += 1;
                self.next += 1;

                Ok(entity)
            },
        }
    }

    /// Return None if the index is invalid.
    /// Use [`try_replace`](IndexMap::try_replace()) if you want to handle the error manually
    #[inline(always)]
    pub fn replace(&mut self, entity: &Entity, data: T) -> Option<T> {
        self.try_replace(entity, data).ok()
    }

    #[inline(always)]
    pub fn try_replace(&mut self, entity: &Entity, data: T) -> Result<T, IndexMapError> {
        match self.inner.get_mut(entity.index()) {
            Some(slot) if entity.version.0 == slot.version => {
                slot.get_content_mut()
                    .ok_or(IndexMapError::InvalidSlot)
                    .map(|prev| core::mem::replace(prev, data))
            },
            _ => Err(IndexMapError::InvalidId),
        }
    }

    #[inline(always)]
    pub fn get(&self, entity: &Entity) -> Option<&T> {
        self.inner
            .get(entity.index())
            .and_then(|slot| {
                if slot.version == entity.version.0 {
                    slot.get_content()
                } else {
                    None
                }
            })
    }

    #[inline(always)]
    pub fn get_mut(&mut self, entity: &Entity) -> Option<&mut T> {
        self.inner
            .get_mut(entity.index())
            .and_then(|slot| {
                if entity.version.0 == slot.version {
                    slot.get_content_mut()
                } else {
                    None
                }
            })
    }

    #[inline(always)]
    pub fn remove(&mut self, entity: &Entity) -> Option<T> {
        self.inner
            .get_mut(entity.index())
            .and_then(|slot| {
                (slot.version == entity.version.0).then_some({
                    slot.try_replace_with(self.next).inspect(|_| {
                        self.next = entity.id.0;
                        self.count -= 1;
                    })
                })?
            })
    }

    #[inline(always)]
    pub fn contains(&self, entity: &Entity) -> bool {
        self.inner
            .get(entity.index())
            .is_some_and(|slot| slot.version == entity.version.0)
    }

    #[inline(always)]
    pub fn len(&self) -> usize { self.count as usize }

    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.count == 0 }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.inner.clear();
        self.inner.push(Slot::new());
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

impl<T> std::ops::Index<Entity> for IndexMap<T> {
    type Output = T;
    fn index(&self, index: Entity) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

#[derive(Debug)]
pub enum IndexMapError {
    ReachedMaxId,
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
mod index_map_test {
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
}
