use std::any::TypeId;

use crate::buffer::TypedErasedBuffer;
use crate::data::component::{Component, ComponentBitset, ComponentBundle, ComponentId};
use crate::data::query::{Query, QueryData};
use crate::entity::Entity;
use crate::map::hash::{TypeIdMap, BitSetMap};
use crate::sparse_set::indices::SparseIndices;

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
    pub(crate) data: TypeIdMap<TypedErasedBuffer>,
    pub(crate) entities: Vec<Entity>,
    pub(crate) indexes: SparseIndices,
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
            data: TypeIdMap::default(),
            entities: Vec::with_capacity(capacity),
            indexes: SparseIndices::default(),
        }
    }

    #[inline(always)]
    pub(crate) fn add_buffer<C: Component>(&mut self, capacity: usize) {
        self.data.insert(C::type_id(), TypedErasedBuffer::with_capacity::<C>(capacity));
    }

    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let buffer = self.data.get_mut(&type_id).unwrap();
        let len = buffer.len();

        buffer.push(component);
        self.indexes.set_index(entity.id(), len);
        self.entities.push(entity);
    }

    // pub fn get_component_buffer<C: Component>(&self, entity: &Entity) -> Option<&TypedErasedBuffer> {}

    pub fn get<C: Component>(&self, entity: &Entity) -> Option<&C> {
        self.indexes
            .get_index(entity.id())
            .and_then(|index| {
                self.version_check(entity, index).then(|| {
                    self.data
                        .get(&C::type_id())
                        .and_then(|buffer| buffer.get(index))
                })?
            })
    }

    pub unsafe fn get_unchecked<C: Component>(&self, entity: &Entity) -> &C {
        todo!()
    }

    #[inline(always)]
    pub fn contains(&self, entity: &Entity) -> bool {
        self.indexes
            .get_index(entity.id())
            .is_some_and(|index| {
                self.entities[index].version() == entity.version()
            })
    }

    fn version_check(&self, entity: &Entity, index: usize) -> bool {
        self.entities[index].version() == entity.version()
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.entities.clear();
        self.indexes.reset();
    }
}

#[derive(Default)]
pub struct ComponentStorage {
    tables: Vec<ComponentTable>,
    table_index: BitSetMap<TableId>,
    component_ids: TypeIdMap<ComponentId>
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            tables: Vec::new(),
            table_index: BitSetMap::default(),
            component_ids: TypeIdMap::default(),
        }
    }

    #[inline(always)]
    pub(crate) fn get_component_id<T: Component>(&self) -> Option<ComponentId> {
        self.component_ids.get(&T::type_id()).copied()
    }

    pub(crate) fn register_component<T: Component>(&mut self) -> ComponentId {
        let type_id = T::type_id();

        if let Some(id) = self.get_component_id::<T>() {
            return id;
        }

        let component_id = ComponentId::new(self.component_ids.len());
        self.component_ids.insert(type_id, component_id);

        component_id
    }

    pub fn registrator(&mut self) -> ComponentRegistrator<'_> {
        ComponentRegistrator {
            storage: self,
            component_bitset: ComponentBitset::new(),
            table: ComponentTable::default()
        }
    }

    pub fn insert_with_table_id<T: Component>(&mut self, id: TableId, entity: Entity, component: T) {
        self.tables[id.index()].insert(entity, component);
    }

    pub fn insert_bundle<B: ComponentBundle>(&mut self, entity: Entity, bundle: B) {
        bundle.insert_bundle(entity, self);
    }

    pub(crate) fn get_table_id_from_bitset(&self, bitset: ComponentBitset) -> Option<TableId> {
        self.table_index.get(&bitset).copied()
    }

    pub fn get_table_id_from_bundle<B: ComponentBundle>(&self) -> Option<TableId> {
        let bitset = B::bitset(self)?;
        self.get_table_id_from_bitset(bitset)
    }
}

pub struct ComponentRegistrator<'a> {
    storage: &'a mut ComponentStorage,
    component_bitset: ComponentBitset,
    table: ComponentTable,
}

impl<'a> ComponentRegistrator<'a> {
    pub(crate) fn register_inner<T: Component>(&mut self, capacity: usize) {
        let component_id = self.storage.register_component::<T>();
        self.component_bitset.update(component_id);
        self.table.add_buffer::<T>(capacity);
    }

    pub fn register<T: Component>(mut self) -> Self {
        self.register_inner::<T>(0);
        self
    }

    pub fn register_with_capacity<T: Component>(mut self, capacity: usize) -> Self {
        self.register_inner::<T>(capacity);
        self
    }

    pub fn finish(self) -> TableId {
        let id = TableId::new(self.storage.tables.len());
        self.storage.table_index.insert(self.component_bitset, id);
        self.storage.tables.push(self.table);
        id
    }
}

/*
#########################################################
#                                                       #
#                         TEST                          #
#                                                       #
#########################################################
*/

#[cfg(test)]
mod component_test {
    use super::*;
    use crate::entity::*;

    #[derive(Debug)] #[allow(unused)] struct Age(u8);
    #[derive(Debug)] #[allow(unused)] struct Name(String);
    #[derive(Debug)] #[allow(unused)] struct Salary(usize);
    #[derive(Debug)] #[allow(unused)] struct Cars(usize);

    #[test]
    fn register_bundle() {
        let mut storage = ComponentStorage::new();
        let mut manager = EntityManager::new();

        for i in 0..10 {
            let entity = manager.create();
            storage.insert_bundle(entity, (Age(i), Name(format!("Balo_{i}"))));
            storage.insert_bundle(entity, (Salary(i as _), Cars(i as _)));
        }

        let table_id = storage.get_table_id_from_bundle::<(Cars, Salary)>().unwrap();
        let table = &storage.tables[table_id.index()];

        let salary_buffer = table.data.get(&Salary::type_id()).unwrap();
        let salary_count = salary_buffer.iter::<Salary>().count();
        assert_eq!(salary_count, 10);

        let age_buffer = table.data.get(&Age::type_id());
        assert!(age_buffer.is_none());

        let non_exist_table = storage.get_table_id_from_bundle::<(Age, Salary, Name)>();
        assert!(non_exist_table.is_none());

        // let vec = salary_buffer.iter::<Salary>().collect::<Vec<_>>();
        // println!("{vec:?}");
    }
}
