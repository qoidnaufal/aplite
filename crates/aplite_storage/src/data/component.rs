use std::{any::TypeId, collections::HashMap};

use super::table::ComponentTable;

use crate::{data::table::TableId, entity::Entity, map::hash::TypeIdMap};

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
// There can only be 64 different components with this current implementation
pub(crate) struct ComponentBitSet(pub(crate) u64);

impl ComponentBitSet {
    pub(crate) fn new() -> Self {
        Self(0)
    }

    pub(crate) fn update(&mut self, id: ComponentId) {
        self.0 |= 1 << id.0
    }
}

impl std::hash::Hash for ComponentBitSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::fmt::Debug for ComponentBitSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ComponentBitSet({:b})", self.0)
    }
}

pub trait Component: Sized + 'static {}

pub trait ComponentBundle {
    type Item;

    fn insert_bundle(self, id: &Entity, table: &mut ComponentTable);
}

pub trait IntoComponent: Sized + 'static {
    type Item: Component;

    fn into_component(self) -> Self::Item;
}

pub struct ComponentStorage {
    tables: Vec<ComponentTable>,
    table_index: HashMap<ComponentBitSet, TableId>,
    component_ids: TypeIdMap<ComponentId>
}

impl ComponentStorage {
    pub(crate) fn register_component<T: Component>(&mut self) -> ComponentId {
        let type_id = TypeId::of::<T>();

        if let Some(id) = self.component_ids.get(&type_id) {
            return *id;
        }

        let component_id = ComponentId::new(self.component_ids.len());
        self.component_ids.insert(type_id, component_id);

        component_id
    }

    pub fn registrator(&mut self) -> ComponentRegister<'_> {
        ComponentRegister {
            storage: self,
            component_bitset: ComponentBitSet::new(),
        }
    }
}

pub struct ComponentRegister<'a> {
    storage: &'a mut ComponentStorage,
    component_bitset: ComponentBitSet,
}

impl<'a> ComponentRegister<'a> {
    pub fn with_component<T: Component>(&mut self, component: T) -> &mut Self {
        let component_id = self.storage.register_component::<T>();
        self.component_bitset.update(component_id);
        self
    }

    pub fn finish(self) {}
}

// Entity   : [0, 1, 2, 3, 4, 5, 6, 7]
// Component: [X, X, X, X, X, X, X, X]

// Entity   : [2, 4, 5, 7]
// Component: [C, C, C, C]

// Entity   : [1, 4, 6]
// Component: [D, D, D]
