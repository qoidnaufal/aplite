use std::any::TypeId;

use crate::data::table::ComponentStorage;
use crate::entity::Entity;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ComponentId(pub(crate) u64);

impl ComponentId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id as _)
    }
}

impl std::hash::Hash for ComponentId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentBitset(pub(crate) u64);

impl ComponentBitset {
    pub(crate) fn new() -> Self {
        Self(0)
    }

    pub(crate) fn update(&mut self, id: ComponentId) {
        self.0 |= 1 << id.0
    }
}

impl std::hash::Hash for ComponentBitset {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::fmt::Debug for ComponentBitset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ComponentBitSet({:b})", self.0)
    }
}

pub trait Component: Sized + 'static {
    fn type_id() -> TypeId {
        TypeId::of::<Self>()
    }
}

impl<T: Sized + 'static> Component for T {}

pub trait ComponentBundle {
    type Item;

    fn insert_bundle(self, id: Entity, storage: &mut ComponentStorage);
    fn bitset(storage: &ComponentStorage) -> Option<ComponentBitset>;
}

// pub trait IntoComponent: Sized + 'static {
//     type Item: Component;

//     fn into_component(self) -> Self::Item;
// }
