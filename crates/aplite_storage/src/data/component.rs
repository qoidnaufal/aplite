use crate::data::table::ComponentStorage;
use crate::entity::Entity;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentId(pub(crate) u64);

impl ComponentId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id as _)
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentBitset(pub(crate) u64);

impl ComponentBitset {
    pub(crate) fn new() -> Self {
        Self(0)
    }

    pub(crate) fn update(&mut self, component_id: ComponentId) {
        self.0 |= 1 << component_id.0
    }

    // pub(crate) fn contains(&self, component_id: ComponentId) -> bool {
    //     self.0 & 1 << component_id.0 == 1 << component_id.0
    // }
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

pub trait Component {}

pub trait ComponentEq: ComponentTuple {
    fn component_eq(&self, other: &Self) -> bool;
}

pub trait ComponentTuple {
    type Item;

    fn insert_bundle(self, entity: Entity, storage: &mut ComponentStorage);
    // fn for_each(&self, f: impl FnMut(&dyn Component));
}

pub(crate) trait ComponentTupleExt {
    fn bitset(storage: &ComponentStorage) -> Option<ComponentBitset>;
}
