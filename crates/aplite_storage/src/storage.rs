use std::marker::PhantomData;
use crate::entity::Entity;
use crate::slot::*;
use crate::Error;

pub struct Storage<E: Entity, T> {
    pub(crate) inner: Vec<Slot<T>>,
    next: u64,
    count: u64,
    marker: PhantomData<E>
}

impl<E: Entity, T> Default for Storage<E, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Entity, T: PartialEq> Storage<E, T> {
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
            return Ok(E::new(idx as u64, slot.version))
        } else {
            self.try_insert(data)
        }
    }
}

impl<E: Entity, T> Storage<E, T> {
    pub fn new() -> Self {
        Self::new_with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::new_with_capacity(capacity)
    }

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

    pub fn insert(&mut self, data: T) -> E {
        self.try_insert(data).unwrap()
    }

    #[inline(always)]
    pub fn try_insert(&mut self, data: T) -> Result<E, Error> {
        if self.count + 1 == u64::MAX { return Err(Error::ReachedMaxId) }

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

    pub fn get(&self, entity: &E) -> Option<&T> {
        self.inner
            .get(entity.index())
            .and_then(|slot| slot.get_content())
    }

    pub fn get_mut(&mut self, entity: &E) -> Option<&mut T> {
        self.inner
            .get_mut(entity.index())
            .and_then(|slot| slot.get_content_mut())
    }

    pub fn unsafe_get(&self, entity: &E) -> Option<&T> {
        self.inner[entity.index()].get_content()
    }

    pub fn unsafe_get_mut(&mut self, entity: &E) -> Option<&mut T> {
        self.inner[entity.index()].get_content_mut()
    }

    #[inline(always)]
    pub fn remove(&mut self, entity: &E) -> Option<T> {
        if let Some(slot) = self.inner.get_mut(entity.index())
        && slot.version == entity.version()
        {
            let ret = std::mem::replace(&mut slot.content, Content::Vacant(self.next));
            slot.version += 1;
            self.next = entity.index() as u64;
            self.count -= 1;
            match ret {
                Content::Occupied(data) => Some(data),
                Content::Vacant(_) => None,
            }
        } else {
            None
        }
    }

    pub fn contains(&self, entity: &E) -> bool {
        self.inner
            .get(entity.index())
            .is_some_and(|slot| slot.version == entity.version())
    }

    pub fn len(&self) -> usize { self.count as usize }

    pub fn is_empty(&self) -> bool { self.count == 0 }

    pub fn iter(&self) -> StorageIterator<'_, E, T> { self.into_iter() }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.inner.push(Slot {
            version: 0,
            content: Content::Vacant(1),
        });
        self.next = 0;
        self.count = 0;
    }
}

impl<'a, E, T> IntoIterator for &'a Storage<E, T>
where
    E: Entity
{
    type Item = (E, &'a T);
    type IntoIter = StorageIterator<'a, E, T>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = self
            .inner
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| {
                slot.get_content()
                    .map(|data| (
                        E::new(i as u64, slot.version),
                        data
                    ))
            })
            .collect::<Vec<_>>();
        StorageIterator {
            inner,
            counter: 0,
        }
    }
}

pub struct StorageIterator<'a, E: Entity, T> {
    inner: Vec<(E, &'a T)>,
    counter: usize,
}

impl<'a, E, T> Iterator for StorageIterator<'a, E, T>
where
    E: Entity,
{
    type Item = (E, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.inner.len() {
            let ret = self.inner[self.counter];
            self.counter += 1;
            Some(ret)
        } else {
            None
        }
    }
}

impl<E, T> std::fmt::Debug for Storage<E, T>
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

#[cfg(test)]
mod storage_iter_test {
    use super::*;
    use crate::entity;

    entity! { TestId }

    #[test]
    fn storage() {
        let mut storage = Storage::<TestId, String>::with_capacity(10);
        let mut created_ids = vec![];

        for i in 0..10 {
            let data = format!("{:#x}", i << 4 | 0x101);
            let id = storage.insert(data);
            created_ids.push(id);
        }

        assert_eq!(storage.len(), 10);
        assert_eq!(created_ids.len(), 10);
        eprintln!("{storage:#?}");

        for i in 0..3 {
            storage.remove(&created_ids[i * 3]);
        }

        assert_eq!(storage.len(), 7);
        eprintln!("{storage:#?}");
    }

    #[test]
    fn no_duplicate() {
        let mut storage = Storage::<TestId, String>::with_capacity(10);
        let mut created_ids = vec![];

        for i in 0..10 {
            let data = if i > 0 && i % 3 == 0 {
                "Double".to_string()
            } else {
                format!("{:#x}", i << 4 | 0x101)
            };
            let id = storage.insert_no_duplicate(data);
            created_ids.push(id);
        }

        assert_eq!(storage.len(), 8);
        assert_eq!(created_ids.len(), 10);
        eprintln!("{storage:#?}");

        for i in 0..3 {
            storage.remove(&created_ids[i * 3]);
        }

        assert_eq!(storage.len(), 6);
        eprintln!("{storage:#?}");
    }
}
