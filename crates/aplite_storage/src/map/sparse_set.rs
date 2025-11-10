use std::iter::Zip;
use std::slice::{Iter, IterMut};

use crate::entity::EntityId;
use crate::data::sparse_index::SparseIndices;

/// A dense data storage which is guaranteed even after removal.
/// Doesn't facilitate the creation of [`Entity`], unlike [`IndexMap`](crate::indexmap::IndexMap).
/// You'll need the assistance of [`EntityManager`](crate::entity::EntityManager) to create the key for indexing data.
pub struct SparseSet<T> {
    pub(crate) data: Vec<T>,
    pub(crate) indexes: SparseIndices,
    pub(crate) entities: Vec<EntityId>,
}

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self {
            data: Vec::default(),
            indexes: SparseIndices::default(),
            entities: Vec::default(),
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for SparseSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter())
            .finish()
    }
}

impl<T> SparseSet<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            indexes: SparseIndices::default(),
            data: Vec::with_capacity(capacity),
            entities: Vec::with_capacity(capacity),
        }
    }

    pub fn reserve_capacity(&mut self, capacity: usize) {
        self.data.reserve_exact(capacity);
        self.entities.reserve_exact(capacity);
    }

    // pub(crate) fn get_raw(&self, entity: &E) -> Option<&UnsafeCell<T>> {
    //     self.ptr
    //         .get_index(entity)
    //         .map(|index| &self.data[index.index()])
    // }

    pub fn get(&self, id: &EntityId) -> Option<&T> {
        self.indexes
            .get_index(id)
            .map(|index| &self.data[index.index()])
    }

    pub fn get_mut(&mut self, id: &EntityId) -> Option<&mut T> {
        self.indexes
            .get_index(id)
            .map(|index| &mut self.data[index.index()])
    }

    pub fn get_or_insert(&mut self, id: &EntityId, value: impl FnOnce() -> T) -> &mut T {
        if let Some(index) = self.indexes.get_index(id)
            && !index.is_null()
        {
            return &mut self.data[index.index()]
        }

        let data_index = self.data.len();

        self.indexes.set_index(id, data_index);
        self.data.push(value());
        self.entities.push(*id);

        self.get_mut(id).unwrap()
    }

    /// Inserting or replacing the value
    pub fn insert(&mut self, id: &EntityId, value: T) {
        if let Some(index) = self.indexes.get_index(id)
            && !index.is_null()
        {
            self.data[index.index()] = value;
            return;
        }

        let data_index = self.data.len();

        self.indexes.set_index(id, data_index);
        self.data.push(value);
        self.entities.push(*id);
    }

    /// The contiguousness of the data is guaranteed after removal via [`Vec::swap_remove`],
    /// but the order of the data is is not.
    pub fn remove(&mut self, id: EntityId) -> Option<T> {
        if self.data.is_empty() { return None }

        self.indexes
            .get_index(&id)
            .cloned()
            .map(|idx| {
                let last = self.entities.last().unwrap();

                self.indexes.set_index(last, idx.index());
                self.indexes.set_null(&id);
                self.entities.swap_remove(idx.index());
                self.data.swap_remove(idx.index())
            })
    }

    /// The length of the data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the data is empty or not
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn contains(&self, id: &EntityId) -> bool {
        self.entities.contains(id)
    }

    pub fn shrink_to_fit(&mut self) {
        self.indexes.shrink_to_fit();
        self.data.shrink_to_fit();
    }

    pub fn reset(&mut self) {
        self.indexes.reset();
        self.data.clear();
    }

    pub fn entity_data_index(&self, id: &EntityId) -> Option<usize> {
        self.indexes.get_index(id).map(|i| i.index())
    }

    pub fn iter(&self) -> ArrayIter<'_, T> {
        ArrayIter::new(self)
    }

    pub fn iter_mut(&mut self) -> ArrayIterMut<'_, T> {
        ArrayIterMut::new(self)
    }

    pub fn iter_data_index(&self) -> impl Iterator<Item = usize> {
        self.indexes.iter_data_index()
    }
}

// impl<E: Entity, T: 'static> Array<E, T> {
//     pub(crate) fn query_one<'a, Q>(&'a self) -> Map<Iter<'a, UnsafeCell<T>>, FnMapQuery<'a, Q>>
//     where
//         Q: QueryData<'a, Item = T>,
//     {
//         self.data.iter().map(map_query::<'a, Q> as FnMapQuery<'a, Q>)
//     }
// }

/*
#########################################################
#                                                       #
#                    Array::Iterator                    #
#                                                       #
#########################################################
*/

pub struct ArrayIter<'a, T> {
    inner: Zip<Iter<'a, EntityId>, Iter<'a, T>>,
}

impl<'a, T> ArrayIter<'a, T> {
    pub(crate) fn new(ds: &'a SparseSet<T>) -> Self {
        let inner = ds.entities
            .iter()
            .zip(ds.data.iter());
        Self {
            inner,
        }
    }
}

impl<'a, T> Iterator for ArrayIter<'a, T> {
    type Item = (&'a EntityId, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub struct ArrayIterMut<'a, T> {
    inner: Zip<Iter<'a, EntityId>, IterMut<'a, T>>,
}

impl<'a, T> ArrayIterMut<'a, T> {
    pub(crate) fn new(ds: &'a mut SparseSet<T>) -> Self {
        let inner = ds.entities
            .iter()
            .zip(ds.data.iter_mut());
        Self {
            inner,
        }
    }
}

impl<'a, T> Iterator for ArrayIterMut<'a, T> {
    type Item = (&'a EntityId, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
