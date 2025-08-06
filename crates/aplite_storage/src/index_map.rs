use std::marker::PhantomData;
use crate::entity::Entity;
use crate::iterator::{IndexMapIter, IndexMapIterMut};
use crate::slot::*;
use crate::Error;

pub struct IndexMap<E: Entity, T> {
    pub(crate) inner: Vec<Slot<T>>,
    next: u32,
    count: u32,
    marker: PhantomData<E>
}

impl<E: Entity, T> Default for IndexMap<E, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Entity, T: PartialEq> IndexMap<E, T> {
    pub fn insert_no_duplicate(&mut self, data: T) -> E {
        self.try_insert_no_duplicate(data).unwrap()
    }

    #[inline(always)]
    pub fn try_insert_no_duplicate(&mut self, data: T) -> Result<E, Error> {
        if let Some((idx, slot)) = self.inner
            .iter()
            .enumerate()
            .find(|(_, slot)| {
                slot.get_content()
                    .is_some_and(|content| content == &data)
            }) {
            return Ok(E::new(idx as u32, slot.version))
        } else {
            self.try_insert(data)
        }
    }
}

impl<E: Entity, T> IndexMap<E, T> {
    #[inline(always)]
    pub fn new() -> Self {
        Self::new_with_capacity(0)
    }

    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::new_with_capacity(capacity)
    }

    /// This would ensure best performance on [`insert`](Self::insert),
    /// but kinda wasteful if you don't really use all the capacity.
    #[inline(always)]
    pub fn with_max_capacity() -> Self {
        let capacity = u64::MAX - 1;
        Self::new_with_capacity(capacity as usize)
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
            marker: PhantomData
        }
    }

    /// Panic if the generated [`Entity`] has reached [`u64::MAX`], or there's an internal error.
    /// Use [`try_insert`](IndexMap::try_insert) if you want to handle the error manually
    #[inline(always)]
    pub fn insert(&mut self, data: T) -> E {
        self.try_insert(data).unwrap()
    }

    #[inline(always)]
    pub fn try_insert(&mut self, data: T) -> Result<E, Error> {
        if self.count + 1 == u32::MAX { return Err(Error::ReachedMaxId) }

        match self.inner.get_mut(self.next as usize) {
            // first time or after removal
            Some(slot) => match slot.content {
                Content::Occupied(_) => return Err(Error::InternalCollision),
                Content::Vacant(idx) => {
                    let entity = E::new(self.next, slot.version);
                    self.next = idx;
                    self.count += 1;
                    slot.content = Content::Occupied(data);

                    Ok(entity)
                },
            }
            None => {
                let entity = Entity::new(self.next, 0);
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

    /// Panic if the id is invalid, or there's an internal error.
    /// Use [`try_replace`](IndexMap::try_replace()) if you want to handle the error manually
    #[inline(always)]
    pub fn replace(&mut self, entity: &E, data: T) -> Option<T> {
        self.try_replace(entity, data).ok()
    }

    #[inline(always)]
    pub fn try_replace(&mut self, entity: &E, data: T) -> Result<T, Error> {
        match self.inner.get_mut(entity.index()) {
            Some(slot) if entity.version() == slot.version => {
                slot.get_content_mut()
                    .ok_or(Error::InvalidSlot)
                    .map(|prev| std::mem::replace(prev, data))
            },
            _ => Err(Error::InvalidId),
        }
    }

    #[inline(always)]
    pub fn get(&self, entity: &E) -> Option<&T> {
        self.inner
            .get(entity.index())
            .and_then(|slot| {
                if slot.version == entity.version() {
                    slot.get_content()
                } else {
                    None
                }
            })
    }

    #[inline(always)]
    pub fn get_mut(&mut self, entity: &E) -> Option<&mut T> {
        self.inner
            .get_mut(entity.index())
            .and_then(|slot| {
                if slot.version == entity.version() {
                    slot.get_content_mut()
                } else {
                    None
                }
            })
    }

    #[inline(always)]
    pub fn remove(&mut self, entity: &E) -> Option<T> {
        if let Some(slot) = self.inner.get_mut(entity.index())
        && slot.version == entity.version()
        {
            let ret = std::mem::replace(&mut slot.content, Content::Vacant(self.next));
            slot.version += 1;
            self.next = entity.index() as u32;
            self.count -= 1;
            match ret {
                Content::Occupied(data) => Some(data),
                Content::Vacant(_) => None,
            }
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn contains(&self, entity: &E) -> bool {
        self.inner
            .get(entity.index())
            .is_some_and(|slot| slot.version == entity.version())
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
    pub fn iter(&self) -> IndexMapIter<'_, E, T> { self.into_iter() }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> IndexMapIterMut<'_, E, T> { self.into_iter() }
}

impl<E, T> std::fmt::Debug for IndexMap<E, T>
where
    E: Entity,
    T: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.iter())
            .finish()
    }
}

impl<E, T> std::ops::Index<&E> for IndexMap<E, T>
where
    E: Entity,
{
    type Output = T;
    fn index(&self, index: &E) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<E, T> std::ops::IndexMut<&E> for IndexMap<E, T>
where
    E: Entity
{
    fn index_mut(&mut self, index: &E) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<E: Entity, T: Clone> Clone for IndexMap<E, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            next: self.next,
            count: self.count,
            marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod index_test {
    use super::*;
    use crate::entity;

    entity! { TestId }

    #[test]
    fn no_duplicate() {
        let mut storage = IndexMap::<TestId, String>::with_capacity(10);
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
