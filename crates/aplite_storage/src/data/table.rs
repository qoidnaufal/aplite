use std::any::TypeId;
use std::collections::HashMap;
use std::ptr::NonNull;

use crate::ArenaPtr;
use crate::buffer::TypeErasedBuffer;
use crate::data::component::{Component, ComponentTuple, ComponentTupleExt, ComponentId};
use crate::data::query::{Query, QueryData, Queryable};
use crate::data::bitset::Bitset;
use crate::entity::EntityId;
use crate::map::hash::TypeIdMap;
use crate::sparse_set::indices::SparseIndices;
use crate::sparse_set::typed::SparseSet;

pub enum Error {
    MaxCapacityReached,
    MismatchedTable,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Error::MaxCapacityReached => "MaxCapacityReached",
            Error::MismatchedTable => "MismatchedTable",
        };

        write!(f, "{msg}")
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for Error {}

impl From<crate::buffer::MaxCapacityReached> for Error {
    fn from(_: crate::buffer::MaxCapacityReached) -> Self {
        Self::MaxCapacityReached
    }
}

/// This is similar to MultiArrayList in Zig, in which Entities with the same composition are stored together.
/// Entity with different composition will produce a different table.
pub struct ArchetypeTable {
    pub(crate) components: SparseSet<ComponentId, TypeErasedBuffer>,

    // Idk if it's going to be safe using only EntityId here
    pub(crate) entities: Vec<EntityId>,

    /// Key is EntityId
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
    pub fn insert<C: Component>(&mut self, component_id: ComponentId, component: C) {
        self.components
            .get_mut(component_id)
            .unwrap()
            .push(component);
    }

    pub fn insert_within_capacity<C: Component>(
        &mut self,
        component_id: ComponentId,
        component: C,
    ) -> Result<ArenaPtr<C>, Error> {
        let table = self.components
            .get_mut(component_id)
            .ok_or(Error::MismatchedTable)?;

        let ptr = table.push_within_capacity(component)?;

        Ok(ptr)
    }

    pub fn get_component_buffer(&self, component_id: ComponentId) -> Option<&TypeErasedBuffer> {
        self.components.get(component_id)
    }

    pub fn get_component_buffer_mut<C: Component>(&mut self, component_id: ComponentId) -> Option<&mut TypeErasedBuffer> {
        self.components.get_mut(component_id)
    }

    pub fn get_component<C: Component>(&self, entity: EntityId, component_id: ComponentId) -> Option<&C> {
        self.indexes
            .get_index(entity)
            .and_then(|index| {
                self.components
                    .get(component_id)
                    .and_then(|buffer| buffer.get(index))
            })
    }

    pub fn get_component_mut<C: Component>(&mut self, entity: EntityId, component_id: ComponentId) -> Option<&mut C> {
        self.indexes
            .get_index(entity)
            .and_then(|index| {
                self.components
                    .get_mut(component_id)
                    .and_then(|buffer| buffer.get_mut(index))
            })
    }

    #[inline(always)]
    pub fn contains(&self, entity: EntityId) -> bool {
        self.indexes
            .get_index(entity)
            .is_some()
    }

    pub fn clear(&mut self) {
        self.components.clear();
        self.entities.clear();
        self.indexes.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableId(pub(crate) usize);

impl TableId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Default)]
pub struct ComponentStorage {
    pub(crate) tables: Vec<ArchetypeTable>,
    pub(crate) table_ids: HashMap<Bitset, TableId>,
    pub(crate) component_ids: TypeIdMap<ComponentId>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            tables: Vec::new(),
            table_ids: HashMap::default(),
            component_ids: TypeIdMap::new(),
        }
    }

    pub(crate) fn get_or_create_id<C: Component + 'static>(&mut self) -> ComponentId {
        if let Some(id) = self.get_component_id::<C>() {
            return id;
        }

        let component_id = ComponentId::new(self.component_ids.len());
        self.component_ids.insert(TypeId::of::<C>(), component_id);

        component_id
    }

    #[inline(always)]
    pub(crate) fn insert<C: Component + 'static>(&mut self, bitset: Bitset, component: C) {
        let component_id = self.component_ids[&TypeId::of::<C>()];
        let table_id = self.table_ids[&bitset];
        let table = &mut self.tables[table_id.0];
        table.insert(component_id, component);
    }

    pub fn insert_component<T: ComponentTuple>(&mut self, entity: EntityId, bundle: T) {
        bundle.insert_bundle(entity, self);
    }

    #[inline(always)]
    pub(crate) fn get_component_id<C: Component + 'static>(&self) -> Option<ComponentId> {
        self.component_ids.get(&TypeId::of::<C>()).copied()
    }

    #[inline(always)]
    pub(crate) fn get_bitset<T>(&self) -> Option<Bitset>
    where
        T: ComponentTuple,
        T::Item: ComponentTupleExt,
    {
        T::Item::bitset(self)
    }

    pub fn get_archetype_table(&self, bitset: Bitset) -> Option<&ArchetypeTable> {
        self.table_ids.get(&bitset).map(|id| &self.tables[id.0])
    }

    pub fn get_archetype_table_mut(&mut self, bitset: Bitset) -> Option<&mut ArchetypeTable> {
        self.table_ids.get(&bitset).map(|id| &mut self.tables[id.0])
    }

    pub(crate) fn get_table_ids<'a>(&'a self, bitset: Bitset) -> Box<[&'a TableId]> {
        self.table_ids
            .keys()
            .filter_map(|bits| {
                bits.contains(&bitset)
                    .then(|| self.table_ids.get(bits))?
            })
            .collect()
    }

    pub fn get_tables<'a>(&'a self, bitset: Bitset) -> Box<[&'a ArchetypeTable]> {
        self.table_ids
            .keys()
            .filter_map(|bits| bits.contains(&bitset)
                .then(|| self.table_ids.get(bits)
                    .map(|id| &self.tables[id.0])
                )?
            )
            .collect()
    }

    pub fn get_queryable_buffers<'a, Q>(&'a self, bitset: Bitset) -> Box<[MarkedBuffer<'a, Q>]>
    where
        Q: Queryable<'a>,
        Q::Item: Component + 'static,
    {
        let component_id = self.get_component_id::<Q::Item>();

        self.table_ids.keys()
            .filter_map(|bits| bits.contains(&bitset)
                .then(|| self.table_ids.get(bits)
                    .map(|id| &self.tables[id.0])
                    .and_then(|table| table.get_component_buffer(component_id?))
                    .map(|buffer| MarkedBuffer {
                        start: buffer.raw.block.cast::<Q::Item>(),
                        len: buffer.len(),
                    })
                )?
            )
            .collect()
    }

    pub(crate) fn get_queryable_buffers_by_id<'a, Q>(&'a self, table_ids: &[&'a TableId]) -> Box<[MarkedBuffer<'a, Q>]>
    where
        Q: Queryable<'a>,
        Q::Item: Component + 'static,
    {
        let component_id = self.get_component_id::<Q::Item>();

        table_ids.iter()
            .filter_map(|id| {
                let table = &self.tables[id.0];
                let buffer = table.get_component_buffer(component_id?)?;

                Some(MarkedBuffer {
                    start: buffer.raw.block.cast::<Q::Item>(),
                    len: buffer.len(),
                })
            })
            .collect()
    }

    pub fn get_marked_buffer<'a, Q>(&'a self, bitset: Bitset) -> Option<MarkedBuffer<'a, Q>>
    where
        Q: Queryable<'a>,
        Q::Item: Component + 'static,
    {
        let component_id = *self.component_ids.get(&TypeId::of::<Q::Item>())?;
        let table_id = self.table_ids.get(&bitset)?;
        let table = &self.tables[table_id.0];
        let buffer = table.get_component_buffer(component_id)?;
        let start = buffer.raw.block.cast::<Q::Item>();
        let len = buffer.len();

        Some(MarkedBuffer {
            start,
            len,
        })
    }

    pub fn archetype_builder(&mut self) -> ArchetypeBuilder<'_> {
        ArchetypeBuilder {
            storage: self,
            bitset: Bitset::new(),
            table: ArchetypeTable::default(),
        }
    }

    pub fn query<'a, Q: QueryData<'a>>(&'a self) -> Query<'a, Q> {
        Query::new(self)
    }
}

pub struct ArchetypeBuilder<'a> {
    storage: &'a mut ComponentStorage,
    bitset: Bitset,
    table: ArchetypeTable,
}

impl<'a> ArchetypeBuilder<'a> {
    #[inline(always)]
    pub(crate) fn register_component<C: Component + 'static>(&mut self, capacity: usize) {
        let component_id = self.storage.get_or_create_id::<C>();
        self.bitset.update(component_id.0);
        self.table.add_buffer::<C>(component_id, capacity);
    }

    pub fn register<T: Component + 'static>(mut self) -> Self {
        self.register_component::<T>(0);
        self
    }

    pub fn register_with_capacity<T: Component + 'static>(mut self, capacity: usize) -> Self {
        self.register_component::<T>(capacity);
        self
    }

    pub fn finish(self) -> Bitset {
        let table_id = TableId::new(self.storage.tables.len());
        self.storage.tables.push(self.table);
        self.storage.table_ids.insert(self.bitset, table_id);
        self.bitset
    }
}

pub struct MarkedBuffer<'a, Q>
where
    Q: Queryable<'a>,
    Q::Item: Component + 'static,
{
    pub(crate) start: NonNull<Q::Item>,
    pub(crate) len: usize,
}

impl<'a, Q> MarkedBuffer<'a, Q>
where
    Q: Queryable<'a>,
    Q::Item: Component + 'static,
{
    pub fn iter(&'a self) -> impl Iterator<Item = Q> {
        MarkedBufferIter {
            start: self.start,
            len: self.len,
            counter: 0,
        }
    }

    pub fn get(&mut self, offset: usize) -> Option<Q> {
        unsafe {
            if offset < self.len {
                let next = self.start.add(offset);
                return Some(Q::convert(next.as_ptr()));
            }

            None
        }
    }
}

pub struct MarkedBufferIter<'a, Q: Queryable<'a>> {
    start: NonNull<Q::Item>,
    len: usize,
    counter: usize,
}

impl<'a, Q> Iterator for MarkedBufferIter<'a, Q>
where
    Q: Queryable<'a>,
    Q::Item: Component + 'static,
{
    type Item = Q;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.counter < self.len {
                let next = self.start.add(self.counter);
                self.counter += 1;
                return Some(Q::convert(next.as_ptr()));
            }

            None
        }
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
    // crate::make_component!(struct Kids((Name, Age)));
    // crate::make_component!(struct Person { name: Name, age: Age });

    #[test]
    fn register_bundle() {
        let mut storage = ComponentStorage::new();
        let mut manager = EntityManager::new();

        let balo = manager.create().id();
        storage.insert_component(balo, (Age(69), Name("Balo".to_string())));
        storage.insert_component(balo, (Salary(6969), Cars(666)));

        let nunez = manager.create().id();
        storage.insert_component(nunez, (Age(69), Name("Balo".to_string())));
    }
}
