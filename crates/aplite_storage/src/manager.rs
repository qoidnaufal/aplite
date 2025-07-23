use crate::iterator::EntityIterator;
use crate::entity::Entity;
use crate::slot::*;
use crate::Error;

pub struct EntityManager<E: Entity> {
    pub(crate) stored: Vec<Slot<E>>,
    next: u64,
    count: u64,
}

impl<E: Entity> Default for EntityManager<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Entity> EntityManager<E> {
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
        let mut stored = Vec::with_capacity(capacity + 1);
        stored.push(Slot {
            version: 0,
            content: Content::Vacant(1),
        });
        Self {
            stored,
            next: 0,
            count: 0,
        }
    }

    pub fn create_entity(&mut self) -> E {
        self.try_create_entity().unwrap()
    }

    #[inline(always)]
    pub fn try_create_entity(&mut self) -> Result<E, Error> {
        if self.count + 1 == u64::MAX { return Err(Error::ReachedMaxId) }

        match self.stored.get_mut(self.next as usize) {
            // first time or after removal
            Some(slot) => match slot.content {
                Content::Occupied(_) => return Err(Error::InternalCollision),
                Content::Vacant(idx) => {
                    let entity = E::new(self.next, slot.version);
                    self.next = idx;
                    self.count += 1;
                    slot.content = Content::Occupied(entity);

                    Ok(entity)
                },
            }
            None => {
                let entity = Entity::new(self.next, 0);
                self.stored.push(Slot {
                    version: 0,
                    content: Content::Occupied(entity),
                });
                self.count += 1;
                self.next += 1;

                Ok(entity)
            },
        }
    }

    pub fn destroy(&mut self, entity: E) {
        if let Some(slot) = self.stored.get_mut(entity.index())
        && slot.version == entity.version()
        {
            slot.content = Content::Vacant(self.next);
            slot.version += 1;
            self.next = entity.index() as u64;
            self.count -= 1;
        }
    }

    #[inline(always)]
    pub fn contains(&self, entity: &E) -> bool {
        self.stored
            .get(entity.index())
            .is_some_and(|slot| slot.version == entity.version())
    }

    #[inline(always)]
    pub fn len(&self) -> usize { self.count as usize }

    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.count == 0 }

    #[inline(always)]
    pub fn get_entities(&self) -> Vec<&E> {
        self.stored
            .iter()
            .filter_map(|slot| slot.get_content())
            .collect::<Vec<_>>()
    }

    #[inline(always)]
    pub fn iter(&self) -> EntityIterator<'_, E> {
        self.into_iter()
    }
}

impl<E: Entity> std::fmt::Debug for EntityManager<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.get_entities().iter())
            .finish()
    }
}

#[cfg(test)]
mod entity_test {
    use super::*;
    use crate::entity;

    entity! { DummyId }

    #[test]
    fn create() {
        let mut manager = EntityManager::<DummyId>::with_capacity(10);
        let mut ids = vec![];
        for _ in 0..10 {
            let id = manager.create_entity();
            ids.push(id);
        }
        eprintln!("{ids:#?}");
        assert_eq!(ids.len(), manager.len())
    }

    #[test]
    fn destroy() {
        let mut manager = EntityManager::<DummyId>::with_capacity(10);
        let mut created_ids = vec![];

        for _ in 0..10 {
            let id = manager.create_entity();
            created_ids.push(id);
        }

        let mut removed = 0;
        for i in 0..created_ids.len() {
            if i > 0 && i % 3 == 0 {
                let to_remove = *created_ids.get(i-1).unwrap();
                manager.destroy(to_remove);
                removed += 1;
            }
        }
        eprintln!("{:?}", manager.stored);
        assert_eq!(created_ids.len() - 3, manager.len());

        let mut new_ids = vec![];
        for _ in 0..removed {
            let new_id = manager.create_entity();
            new_ids.push(new_id);
        }

        eprintln!("{created_ids:#?}");
        assert!(new_ids.iter().all(|id| id.version() > 0))
    }

    #[test]
    fn macro_test() {
        entity! {
            One,
            Two,
            Three
        }

        let mut one = EntityManager::<One>::new();
        let mut two = EntityManager::<Two>::new();
        let mut three = EntityManager::<Three>::new();

        let id_one = one.create_entity();
        let id_two = two.create_entity();
        let id_three = three.create_entity();

        assert_eq!(id_one.index(), id_two.index());
        assert_eq!(id_two.version(), id_three.version());
    }
}

// mod alt {
//     use super::{Slot, Content};
//     use crate::Entity;
//     use crate::Error;

//     struct Manager<E: Entity> {
//         reusable: Vec<E>,
//         next: u64,
//     }

//     impl<E: Entity> Manager<E> {
//         fn try_create_entity(&mut self) -> Result<E, Error> {
//             match self.reusable.pop() {
//                 Some(e) => Ok(e),
//                 None => {
//                     if self.next + 1 == u64::MAX { return Err(Error::ReachedMaxId) }
//                     let e = E::new(self.next, 0);
//                     self.next += 1;
//                     Ok(e)
//                 },
//             }
//         }

//         fn destroy(&mut self, entity: E) {
//             let reuse = E::new(entity.index() as u64, entity.version() + 1);
//             self.reusable.push(reuse);
//         }
//     }
// }
