use crate::iterator::EntityIterator;

/// A trait that needs to be implemented for any type to be stored in the [`Tree`]
pub trait Entity
where
    Self : std::fmt::Debug + Copy + PartialEq + PartialOrd
{
    /// If you created this manually, you also need to manually [`insert()`](crate::tree::Tree::insert) it to the [`Tree`](crate::tree::Tree).
    /// The [`Tree`](crate::tree::Tree) provides a hassle free [`create_entity()`](Tree::create_entity) method
    /// to create an [`Entity`] and automatically insert it.
    fn new(index: u64, version: u32) -> Self;

    /// The index where this [`Entity`] is being stored inside the [`Tree`]
    fn index(&self) -> usize;

    fn version(&self) -> u32;
}

/// A macro to confeniently implement [`Entity`] trait to be stored in the [`Tree`].
/// You just need to specify the name.
/// # Example
/// ```ignore
/// entity! {
///     SuperUniqueIdName;
///     AnotherId;
/// }
///
/// let mut tree: Tree<SuperUniqueIdName> = Tree::new();
/// let super_unique_id_name: SuperUniqueIdName = tree.create_entity();
/// let another_id = AnotherId::new();
/// ```
#[macro_export]
macro_rules! entity {
    { $vis:vis $name:ident } => {
        #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        $vis struct $name(u64, u32);

        impl Entity for $name {
            fn new(index: u64, version: u32) -> Self {
                Self(index, version)
            }

            fn index(&self) -> usize {
                self.0 as usize
            }

            fn version(&self) -> u32 {
                self.1
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl std::hash::Hash for $name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                state.write_u64(self.0);
            }
        }
    };

    { $vis:vis $name:ident, } => {
        entity! { $vis $name }
    };

    { $vis:vis $name:ident, $($vis2:vis $name2:ident),* } => {
        entity! { $vis $name }
        entity! { $($vis2 $name2),* }
    };

    { $vis:vis $name:ident, $($vis2:vis $name2:ident),*, } => {
        entity! { $vis $name }
        entity! { $($vis2 $name2),* }
    };
}

#[derive(Debug)]
pub enum Error {
    ReachedMaxId,
    InternalCollision,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) enum Content<T> {
    Occupied(T),
    // contains index of next free slot
    Vacant(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct Slot<T> {
    pub(crate) version: u32,
    content: Content<T>,
}

impl<T> Slot<T> {
    #[inline(always)]
    pub(crate) fn vacant(pref_free_slot: u64) -> Self {
        Self {
            content: Content::Vacant(pref_free_slot),
            version: 0,
        }
    }

    #[inline(always)]
    pub(crate) fn occupied(val: T) -> Self {
        Self {
            content: Content::Occupied(val),
            version: 0,
        }
    }

    pub(crate) fn get_content(&self) -> Option<&T> {
        match &self.content {
            Content::Occupied(val) => Some(val),
            Content::Vacant(_) => None,
        }
    }
}

#[derive(Clone)]
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
        let mut inner = Vec::with_capacity(capacity + 1);
        let slot = Slot::vacant(1);
        inner.push(slot);
        Self {
            stored: inner,
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
                self.stored.push(Slot::occupied(entity));
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
