use crate::data::table::ComponentStorage;
use crate::data::bitset::Bitset;
use crate::entity::EntityId;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentId(pub(crate) usize);

impl ComponentId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }
}

impl std::fmt::Debug for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ComponentId({})", self.0)
    }
}

impl std::hash::Hash for ComponentId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub trait Component {}

pub trait ComponentEq: ComponentTuple {
    fn component_eq(&self, other: &Self) -> bool;
}

pub trait ComponentTuple {
    type Item;

    fn insert_bundle(self, entity: EntityId, storage: &mut ComponentStorage);
}

pub(crate) trait ComponentTupleExt {
    fn bitset(storage: &ComponentStorage) -> Option<Bitset>;
}
