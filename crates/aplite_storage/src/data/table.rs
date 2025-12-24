use std::any::TypeId;

use crate::buffer::TypeErasedBuffer;
use crate::data::component::{Component, ComponentBitset, ComponentTuple, ComponentTupleExt, ComponentId};
// use crate::data::query::{Query, QueryData};
use crate::entity::Entity;
use crate::map::hash::{TypeIdMap, BitSetMap};
use crate::sparse_set::indices::SparseIndices;
use crate::sparse_set::type_erased::TypeErasedSparseSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableId(usize);

impl TableId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }

    pub(crate) fn index(&self) -> usize {
        self.0
    }
}

/// This is similar to MultiArrayList in Zig, in which Entities with the same composition are stored together.
/// Entity with different composition will produce a new ComponentTable.
pub struct ComponentTable {
    /// K: ComponentId, V: TypeErasedBuffer
    pub(crate) data: TypeErasedSparseSet,

    // Idk if it's going to be safe using only EntityId here
    pub(crate) entities: Vec<Entity>,

    /// Key is either Entity or EntityId
    pub(crate) indexes: SparseIndices,
}

impl std::fmt::Debug for ComponentTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.indexes.iter_data_index().zip(self.entities.iter()))
            .finish()
    }
}

impl Default for ComponentTable {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl ComponentTable {
    /// Set the capacity for the contained entity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: TypeErasedSparseSet::new::<TypeErasedBuffer>(),
            entities: Vec::with_capacity(capacity),
            indexes: SparseIndices::default(),
        }
    }

    #[inline(always)]
    pub(crate) fn add_buffer<C: Component>(&mut self, component_id: ComponentId, capacity: usize) {
        self.data.insert(component_id, TypeErasedBuffer::with_capacity::<C>(capacity));
    }

    #[inline(always)]
    pub(crate) fn insert<C: Component>(&mut self, component: C, component_id: ComponentId) {
        self.data
            .get_mut::<ComponentId, TypeErasedBuffer>(component_id)
            .unwrap()
            .push(component);
    }

    pub fn get_component_buffer<C: Component>(&self, component_id: ComponentId) -> Option<&TypeErasedBuffer> {
        self.data.get(component_id)
    }

    pub fn get_component_buffer_mut<C: Component>(&mut self, component_id: ComponentId) -> Option<&mut TypeErasedBuffer> {
        self.data.get_mut(component_id)
    }

    pub fn get_component<C: Component>(&self, entity: Entity, component_id: ComponentId) -> Option<&C> {
        self.indexes
            .get_index(entity.index())
            .and_then(|index| {
                self.version_check(entity, index).then(|| {
                    self.data
                        .get::<ComponentId, TypeErasedBuffer>(component_id)
                        .and_then(|buffer| buffer.get(index))
                })?
            })
    }

    pub fn get_component_mut<C: Component>(&mut self, entity: Entity, component_id: ComponentId) -> Option<&mut C> {
        self.indexes
            .get_index(entity.index())
            .and_then(|index| {
                self.version_check(entity, index).then(|| {
                    self.data
                        .get_mut::<ComponentId, TypeErasedBuffer>(component_id)
                        .and_then(|buffer| buffer.get_mut(index))
                })?
            })
    }

    #[inline(always)]
    pub fn contains(&self, entity: Entity) -> bool {
        self.indexes
            .get_index(entity.index())
            .is_some_and(|index| {
                self.entities[index].version() == entity.version()
            })
    }

    fn version_check(&self, entity: Entity, index: usize) -> bool {
        #[cfg(debug_assertions)]
        {
            self.entities[index].version() == entity.version()
        }

        #[cfg(not(debug_assertions))]
        {
            true
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.entities.clear();
        self.indexes.clear();
    }
}

#[derive(Default)]
pub struct ComponentStorage {
    pub(crate) tables: Vec<ComponentTable>,
    pub(crate) table_ids: BitSetMap<TableId>,
    pub(crate) component_ids: TypeIdMap<ComponentId>
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            tables: Vec::new(),
            table_ids: BitSetMap::default(),
            component_ids: TypeIdMap::default(),
        }
    }

    pub(crate) fn create_component_id<C: Component + 'static>(&mut self) -> ComponentId {
        if let Some(id) = self.get_component_id::<C>() {
            return id;
        }

        let component_id = ComponentId::new(self.component_ids.len());
        self.component_ids.insert(TypeId::of::<C>(), component_id);

        component_id
    }

    #[inline(always)]
    pub(crate) fn get_component_id<C: Component + 'static>(&self) -> Option<ComponentId> {
        self.component_ids.get(&TypeId::of::<C>()).copied()
    }

    #[inline(always)]
    pub fn insert_with_table_id<C: Component + 'static>(&mut self, table_id: TableId, component: C) {
        let component_id = self.component_ids[&TypeId::of::<C>()];
        self.tables[table_id.index()].insert(component, component_id);
    }

    pub fn insert_component_tuple<T: ComponentTuple>(&mut self, entity: Entity, bundle: T) {
        bundle.insert_bundle(entity, self);
    }

    // used in component_bundle! macro
    pub(crate) fn get_table_mut_from_table_id(&mut self, table_id: TableId) -> &mut ComponentTable {
        &mut self.tables[table_id.index()]
    }

    // pub(crate) fn get_table_mut_from_bundle<T>(&mut self) -> Option<&mut ComponentTable>
    // where
    //     T: ComponentTuple,
    //     T::Item: ComponentTupleExt,
    // {
    //     self.get_table_id_from_bundle::<T>()
    //         .map(|table_id| &mut self.tables[table_id.index()])
    // }

    #[inline(always)]
    pub(crate) fn get_table_id_from_bundle<T>(&self) -> Option<TableId>
    where
        T: ComponentTuple,
        T::Item: ComponentTupleExt,
    {
        self.table_ids.get(&self.get_bitset::<T>()?).copied()
    }

    #[inline(always)]
    pub(crate) fn get_bitset<T>(&self) -> Option<ComponentBitset>
    where
        T: ComponentTuple,
        T::Item: ComponentTupleExt,
    {
        T::Item::bitset(self)
    }

    pub fn registrator(&mut self) -> ComponentRegistrator<'_> {
        ComponentRegistrator {
            storage: self,
            component_bitset: ComponentBitset::new(),
            table: ComponentTable::default()
        }
    }
}

pub struct ComponentRegistrator<'a> {
    storage: &'a mut ComponentStorage,
    component_bitset: ComponentBitset,
    table: ComponentTable,
}

impl<'a> ComponentRegistrator<'a> {
    #[inline(always)]
    pub(crate) fn register_component<T: Component + 'static>(&mut self, capacity: usize) {
        let component_id = self.storage.create_component_id::<T>();
        self.component_bitset.update(component_id);
        self.table.add_buffer::<T>(component_id, capacity);
    }

    pub fn register<T: Component + 'static>(mut self) -> Self {
        self.register_component::<T>(0);
        self
    }

    pub fn register_with_capacity<T: Component + 'static>(mut self, capacity: usize) -> Self {
        self.register_component::<T>(capacity);
        self
    }

    pub fn finish(self) -> TableId {
        let id = TableId::new(self.storage.tables.len());
        self.storage.table_ids.insert(self.component_bitset, id);
        self.storage.tables.push(self.table);
        id
    }
}

/*
#########################################################
#
# TEST
#
#########################################################
*/

#[cfg(test)]
mod component_test {
    use super::*;
    use crate::entity::*;

    crate::make_component!(struct Age(u8));
    crate::make_component!(struct Name(String));
    crate::make_component!(struct Salary(usize));
    crate::make_component!(struct Cars(usize));
    crate::make_component!(struct Kids((Name, Age)));
    crate::make_component!(struct Person { name: Name, age: Age });

    #[test]
    fn register_bundle() {
        let mut storage = ComponentStorage::new();
        let mut manager = EntityManager::new();

        let entity = manager.create();
        storage.insert_component_tuple(entity, (Age(69), Name("Balo".to_string())));
        storage.insert_component_tuple(entity, (Salary(6969), Cars(666)));

        let component_id = storage.get_component_id::<Cars>();
        assert!(component_id.is_some());

        let table_id = storage.get_table_id_from_bundle::<(Cars, Kids, Person)>();
        assert!(table_id.is_none());
    }
}
