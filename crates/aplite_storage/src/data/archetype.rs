use aplite_bitset::Bitset;

use crate::entity::EntityId;
use crate::sparse_set::indices::SparseIndices;
use crate::sparse_set::SparseSet;
use crate::arena::ptr::ArenaPtr;
use crate::buffer::TypeErasedBuffer;
use crate::data::component::{ComponentId, Component};
use crate::data::component_storage::ComponentStorage;

pub enum Error {
    MaxCapacityReached,
    MismatchedTable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchetypeId(pub(crate) usize);

pub struct ArchetypeBuilder<'a> {
    pub(crate) storage: &'a mut ComponentStorage,
    pub(crate) bitset: Bitset,
    pub(crate) table: ArchetypeTable,
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

/*
#########################################################
#
# impl ArchetypeTable
#
#########################################################
*/

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

/*
#########################################################
#
# impl ArchetypeBuilder
#
#########################################################
*/

impl<'a> ArchetypeBuilder<'a> {
    #[inline(always)]
    pub(crate) fn register_component<C: Component + 'static>(&mut self, capacity: usize) {
        let component_id = self.storage.get_or_create_id::<C>();
        self.bitset.add_bit(component_id.0);
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

    pub fn finish(self) -> ArchetypeId {
        let id = ArchetypeId(self.storage.archetype_tables.len());
        self.storage.archetype_tables.push(self.table);
        self.storage.archetype_ids.insert(self.bitset, id);
        id
    }
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
