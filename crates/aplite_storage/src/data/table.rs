use std::any::TypeId;
use std::ptr::NonNull;

use crate::buffer::TypeErasedBuffer;
use crate::data::component::{Component, ComponentTuple, ComponentId};
use crate::data::query::{Query, QueryData, Queryable};
use crate::entity::Entity;
use crate::map::hash::TypeIdMap;
use crate::sparse_set::SparsetKey;
use crate::sparse_set::indices::SparseIndices;
use crate::sparse_set::typed::SparseSet;

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
/// Entity with different composition will produce a different table.
pub struct ArchetypeTable {
    pub(crate) components: SparseSet<ComponentId, TypeErasedBuffer>,

    // Idk if it's going to be safe using only EntityId here
    pub(crate) entities: Vec<Entity>,

    /// Key is either Entity or EntityId
    pub(crate) indexes: SparseIndices,
}

impl std::fmt::Debug for ArchetypeTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.indexes.iter_data_index().zip(self.entities.iter()))
            .finish()
    }
}

impl Default for ArchetypeTable {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl ArchetypeTable {
    /// Set the capacity for the contained entity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            components: SparseSet::new(),
            entities: Vec::with_capacity(capacity),
            indexes: SparseIndices::default(),
        }
    }

    #[inline(always)]
    pub fn add_buffer<C: Component>(&mut self, component_id: ComponentId, capacity: usize) {
        self.components.insert(component_id, TypeErasedBuffer::with_capacity::<C>(capacity));
    }

    #[inline(always)]
    pub fn insert<C: Component>(&mut self, component: C, component_id: ComponentId) {
        self.components
            .get_mut(component_id)
            .unwrap()
            .push(component);
    }

    pub fn get_component_buffer(&self, component_id: ComponentId) -> Option<&TypeErasedBuffer> {
        self.components.get(component_id)
    }

    pub fn get_component_buffer_mut<C: Component>(&mut self, component_id: ComponentId) -> Option<&mut TypeErasedBuffer> {
        self.components.get_mut(component_id)
    }

    pub fn get_component<C: Component>(&self, entity: Entity, component_id: ComponentId) -> Option<&C> {
        self.indexes
            .get_index(entity)
            .and_then(|index| {
                self.version_check(entity, index).then(|| {
                    self.components
                        .get(component_id)
                        .and_then(|buffer| buffer.get(index))
                })?
            })
    }

    pub fn get_component_mut<C: Component>(&mut self, entity: Entity, component_id: ComponentId) -> Option<&mut C> {
        self.indexes
            .get_index(entity)
            .and_then(|index| {
                self.version_check(entity, index).then(|| {
                    self.components
                        .get_mut(component_id)
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
        self.components.clear();
        self.entities.clear();
        self.indexes.clear();
    }
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

    pub fn get_marked_buffer_with_entities<'a, Q>(
        &'a self,
        entities: &'a [Entity]
    ) -> Option<MarkedBuffer<'a, Q>>
    where
        Q: Queryable<'a>,
        Q::Item: Component + 'static,
    {
        self.get_component_id::<Q::Item>()
            .map(|component_id| {
                let c_index = component_id.index();
                let raw = self.components[c_index].raw.block.cast::<Q::Item>();

                MarkedBuffer {
                    entities,
                    indices: &self.indexer[c_index].indices,
                    raw,
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
}

impl TableIndexer {
    fn new(capacity: usize) -> Self {
        Self {
            entities: Vec::with_capacity(capacity),
            indices: SparseIndices::new(),
        }
    }
}

pub struct MarkedBuffer<'a, Q>
where
    Q: Queryable<'a>,
    Q::Item: Component + 'static,
{
    pub(crate) entities: &'a [Entity],
    pub(crate) indices: &'a SparseIndices,
    pub(crate) raw: NonNull<Q::Item>,
}

impl<'a, Q> MarkedBuffer<'a, Q>
where
    Q: Queryable<'a>,
    Q::Item: Component + 'static,
{
    pub fn iter(&'a self) -> impl Iterator<Item = Q> {
        MarkedBufferIter {
            raw: self.raw,
            entities: self.entities,
            indices: self.indices,
            cursor: 0,
        }
    }

    pub fn get(&self, entity: Entity) -> Option<Q> {
        if !self.entities.contains(&entity) {
            return None;
        }

        let index = self.indices.get_index(entity)?;

        unsafe {
            Some(Q::convert(self.raw.add(index).as_ptr()))
        }
    }
}

pub struct MarkedBufferIter<'a, Q: Queryable<'a>> {
    raw: NonNull<Q::Item>,
    pub(crate) entities: &'a [Entity],
    pub(crate) indices: &'a SparseIndices,
    cursor: usize,
}

impl<'a, Q> Iterator for MarkedBufferIter<'a, Q>
where
    Q: Queryable<'a>,
    Q::Item: Component + 'static,
{
    type Item = Q;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let entity = *self.entities.get(self.cursor)?;
            let index = self.indices.get_index(entity)?;
            self.cursor += 1;
            Some(Q::convert(self.raw.add(index).as_ptr()))
        }
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
