use crate::entity::Entity;
use super::sparse_index::DataPointer;
use crate::iterator::{
    DataStoreIter,
    DataStoreIterMut,
    MappedDataStoreIter,
};

/// A dense data storage which is guaranteed even after removal.
/// Doesn't facilitate the creation of [`Entity`], unlike [`IndexMap`](crate::index_map::IndexMap).
/// You'll need the assistance of [`EntityManager`](crate::entity::EntityManager) to create the key for indexing data.
pub struct DenseRow<E: Entity, T> {
    pub(crate) ptr: DataPointer<E>,
    pub(crate) data: Vec<T>,
}

impl<E: Entity, T> Default for DenseRow<E, T> {
    fn default() -> Self {
        Self {
            ptr: DataPointer::default(),
            data: Vec::default(),
        }
    }
}

impl<E: crate::Entity, T: std::fmt::Debug> std::fmt::Debug for DenseRow<E, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter())
            .finish()
    }
}

impl<E: Entity, T> DenseRow<E, T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            ptr: DataPointer::default(),
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    pub fn get(&self, entity: E) -> Option<&T> {
        self.ptr
            .with(entity, |index| &self.data[index])
    }

    pub fn get_mut(&mut self, entity: E) -> Option<&mut T> {
        self.ptr
            .with(entity, |index| &mut self.data[index])
    }

    /// Inserting or replacing the value
    pub fn insert(&mut self, entity: E, value: T) {
        self.ptr.insert(entity, value, &mut self.data);
    }

    /// The contiguousness of the data is guaranteed after removal via [`Vec::swap_remove`],
    /// but the order of the data is is not.
    pub fn remove(&mut self, entity: E) -> Option<T> {
        self.ptr.remove(entity, &mut self.data)
    }

    /// The length of the data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the data is empty or not
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn contains(&self, entity: E) -> bool {
        self.ptr.contains(entity)
    }

    pub fn reset(&mut self) {
        self.ptr.reset();
        self.data.clear();
    }

    pub fn entity_data_index(&self, entity: E) -> Option<usize> {
        self.ptr.entity_data_index(entity)
    }

    pub fn drain_all(&mut self) -> std::vec::Drain<'_, T> {
        self.ptr.reset();
        self.data.drain(..)
    }

    pub fn iter(&self) -> DataStoreIter<'_, T> {
        DataStoreIter::new(self)
    }

    pub fn iter_mut(&mut self) -> DataStoreIterMut<'_, T> {
        DataStoreIterMut::new(self)
    }

    pub fn iter_data_index(&self) -> impl Iterator<Item = &usize> {
        self.ptr.iter_data_index()
    }

    pub fn iter_map(&self) -> MappedDataStoreIter<'_, E, T> {
        MappedDataStoreIter::new(self)
    }
}
