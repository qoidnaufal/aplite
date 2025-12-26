use std::any::TypeId;

use crate::buffer::TypeErasedBuffer;
use crate::data::component::{Component, ComponentTuple, ComponentId};
use crate::data::query::{Query, QueryData, Queryable};
use crate::entity::Entity;
use crate::map::hash::TypeIdMap;
use crate::sparse_set::SparsetKey;
use crate::sparse_set::indices::SparseIndices;
use crate::sparse_set::type_erased::TypeErasedSparseSet;

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct TableId(usize);

// impl TableId {
//     pub(crate) fn new(id: usize) -> Self {
//         Self(id)
//     }

//     pub(crate) fn index(&self) -> usize {
//         self.0
//     }
// }

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

    // #[inline(always)]
    // pub(crate) fn add_buffer<C: Component>(&mut self, component_id: ComponentId, capacity: usize) {
    //     self.data.insert(component_id, TypeErasedBuffer::with_capacity::<C>(capacity));
    // }

    // #[inline(always)]
    // pub(crate) fn insert<C: Component>(&mut self, component: C, component_id: ComponentId) {
    //     self.data
    //         .get_mut::<ComponentId, TypeErasedBuffer>(component_id)
    //         .unwrap()
    //         .push(component);
    // }

    pub fn get_component_buffer(&self, component_id: ComponentId) -> Option<&TypeErasedBuffer> {
        self.data.get(component_id)
    }

    pub fn get_component_buffer_mut<C: Component>(&mut self, component_id: ComponentId) -> Option<&mut TypeErasedBuffer> {
        self.data.get_mut(component_id)
    }

    pub fn get_component<C: Component>(&self, entity: Entity, component_id: ComponentId) -> Option<&C> {
        self.indexes
            .get_index(entity)
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
            .get_index(entity)
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
            .get_index(entity)
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

pub struct MarkedBuffer<'a, Q> {
    pub(crate) indexes: &'a SparseIndices,
    pub(crate) buffer: &'a TypeErasedBuffer,
    marker: std::marker::PhantomData<Q>
}

#[derive(Default, Clone)]
pub(crate) struct TableIndexer {
    pub(crate) entities: Vec<Entity>,
    pub(crate) indices: SparseIndices,
}

#[derive(Default)]
pub struct ComponentStorage {
    pub(crate) components: Vec<TypeErasedBuffer>,
    pub(crate) indexer: Vec<TableIndexer>,
    pub(crate) component_ids: TypeIdMap<ComponentId>,

    // pub(crate) tables: Vec<ComponentTable>,
    // pub(crate) table_ids: BitSetMap<TableId>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            indexer: Vec::new(),
            component_ids: TypeIdMap::default(),

            // archetypes: BitSetMap::default(),
            // tables: Vec::new(),
            // table_ids: BitSetMap::default(),
        }
    }

    #[inline(always)]
    pub(crate) fn get_or_create_id_with_capacity<C>(&mut self, capacity: usize) -> ComponentId
    where
        C: Component + 'static,
    {
        if let Some(id) = self.get_component_id::<C>() {
            return id;
        }

        // let component_id = ComponentId::new(self.component_ids.len());
        let component_id = ComponentId::new(self.components.len());
        self.component_ids.insert(TypeId::of::<C>(), component_id);

        self.components.push(TypeErasedBuffer::with_capacity::<C>(capacity)); // new
        self.indexer.push(TableIndexer::new(capacity)); // new

        component_id
    }

    pub(crate) fn get_or_create_id<C: Component + 'static>(&mut self) -> ComponentId {
        self.get_or_create_id_with_capacity::<C>(0)
    }


    #[inline(always)]
    pub fn insert<C: Component + 'static>(&mut self, entity: Entity, component: C) {
        let index = self.get_or_create_id::<C>().index();
        let buffer = &mut self.components[index];
        let TableIndexer { entities, indices } = &mut self.indexer[index];
        let len = buffer.len();

        buffer.push(component);
        indices.set_index(entity, len);
        entities.push(entity);
    }

    pub fn insert_component_tuple<T: ComponentTuple>(&mut self, entity: Entity, bundle: T) {
        bundle.insert_bundle(entity, self);
    }

    #[inline(always)]
    pub(crate) fn get_component_id<C: Component + 'static>(&self) -> Option<ComponentId> {
        self.component_ids.get(&TypeId::of::<C>()).copied()
    }

    pub fn get_component_buffer<C: Component + 'static>(&self) -> Option<&TypeErasedBuffer> {
        self.get_component_id::<C>()
            .map(|component_id| &self.components[component_id.index()])
    }

    pub fn get_marked_buffer<Q>(&self) -> Option<MarkedBuffer<'_, Q>>
    where
        Q: Queryable,
        Q::Item: Component + 'static,
    {
        self.get_component_id::<Q::Item>()
            .map(|component_id| {
                MarkedBuffer {
                    indexes: &self.indexer[component_id.index()].indices,
                    buffer: &self.components[component_id.index()],
                    marker: std::marker::PhantomData,
                }
            })
    }

    pub fn get_component<C: Component + 'static>(&self, entity: Entity) -> Option<&C> {
        self.get_component_id::<C>()
            .and_then(|component_id| {
                let buffer = &self.components[component_id.index()];
                let indexer = &self.indexer[component_id.index()];
                indexer.indices.get_index(entity)
                    .map(|index| unsafe { buffer.get_unchecked::<C>(index) })
            })
    }

    pub fn get_entities<C: Component + 'static>(&self) -> Vec<Entity> {
        self.get_component_id::<C>()
            .map(|component_id| {
                self.indexer[component_id.0 as usize]
                    .entities
                    .clone()
            })
            .unwrap_or(vec![])
    }

    pub fn query<'a, Q: QueryData<'a>>(&'a self) -> Query<'a, Q> {
        Query::new(self)
    }

    // #[inline(always)]
    // pub(crate) fn get_bitset<T>(&self) -> Option<ComponentBitset>
    // where
    //     T: ComponentTuple,
    //     T::Item: ComponentTupleExt,
    // {
    //     T::Item::bitset(self)
    // }

    // #[inline(always)]
    // pub fn insert_with_table_id<C: Component + 'static>(&mut self, table_id: TableId, component: C) {
    //     let component_id = self.component_ids[&TypeId::of::<C>()];
    //     self.tables[table_id.index()].insert(component, component_id);
    // }

    // used in component_bundle! macro
    // pub(crate) fn get_table_mut_from_table_id(&mut self, table_id: TableId) -> &mut ComponentTable {
    //     &mut self.tables[table_id.index()]
    // }

    // pub(crate) fn get_table_mut_from_bundle<T>(&mut self) -> Option<&mut ComponentTable>
    // where
    //     T: ComponentTuple,
    //     T::Item: ComponentTupleExt,
    // {
    //     self.get_table_id_from_bundle::<T>()
    //         .map(|table_id| &mut self.tables[table_id.index()])
    // }

    // #[inline(always)]
    // pub(crate) fn get_table_id_from_bundle<T>(&self) -> Option<TableId>
    // where
    //     T: ComponentTuple,
    //     T::Item: ComponentTupleExt,
    // {
    //     self.table_ids.get(&self.get_bitset::<T>()?).copied()
    // }

    // pub fn archetype_builder(&mut self) -> ArchetypeBuilder<'_> {
    //     ArchetypeBuilder {
    //         storage: self,
    //         component_bitset: ComponentBitset::new(),
    //     }
    // }
}

impl TableIndexer {
    fn new(capacity: usize) -> Self {
        Self {
            entities: Vec::with_capacity(capacity),
            indices: SparseIndices::new(),
        }
    }
}

impl<'a, Q: Queryable + 'static> MarkedBuffer<'a, Q> {
    pub fn iter(&self) -> impl Iterator<Item = Q> {
        self.buffer.iter_raw::<Q::Item>()
            .map(|raw| Q::convert(raw))
    }

    pub fn get_component(&self, entity: Entity) -> Option<Q> {
        self.indexes.get_index(entity)
            .map(|index| unsafe {
                let raw = self.buffer.get_unchecked_raw(index);
                Q::convert(raw)
            })
    }
}

// pub struct ArchetypeBuilder<'a> {
//     storage: &'a mut ComponentStorage,
//     component_bitset: ComponentBitset,
// }

// impl<'a> ArchetypeBuilder<'a> {
//     #[inline(always)]
//     pub(crate) fn register_component<C: Component + 'static>(&mut self, capacity: usize) {
//         let component_id = self.storage.get_or_create_id_with_capacity::<C>(capacity);
//         self.component_bitset.update(component_id);

//         // let component_id = self.storage.get_or_create_component_id::<T>();
//         // self.component_bitset.update(component_id);
//         // self.table.add_buffer::<T>(component_id, capacity);
//     }

//     pub fn register<T: Component + 'static>(mut self) -> Self {
//         self.register_component::<T>(0);
//         self
//     }

//     pub fn register_with_capacity<T: Component + 'static>(mut self, capacity: usize) -> Self {
//         self.register_component::<T>(capacity);
//         self
//     }

//     pub fn finish(self) -> ComponentBitset {
//         self.component_bitset
//     }
// }

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
    // crate::make_component!(struct Kids((Name, Age)));
    // crate::make_component!(struct Person { name: Name, age: Age });

    #[test]
    fn register_bundle() {
        let mut storage = ComponentStorage::new();
        let mut manager = EntityManager::new();

        let balo = manager.create();
        storage.insert_component_tuple(balo, (Age(69), Name("Balo".to_string())));
        storage.insert_component_tuple(balo, (Salary(6969), Cars(666)));

        let nunez = manager.create();
        storage.insert_component_tuple(nunez, (Age(69), Name("Balo".to_string())));

        assert_eq!(storage.components.len(), 4);

        let cars_id = storage.get_component_id::<Cars>();
        assert!(cars_id.is_some());

        let entities = storage.get_entities::<Age>();
        assert_eq!(entities.len(), 2);

        let age_len = storage.get_component_buffer::<Age>().unwrap().len();
        let salary_len = storage.get_component_buffer::<Salary>().unwrap().len();
        assert!(age_len > salary_len);
    }
}
