use std::iter::Zip;
use std::slice::{Iter, IterMut};

use crate::entity::Entity;
use super::sparse_index::SparseIndices;

/// A dense data storage which is guaranteed even after removal.
/// Doesn't facilitate the creation of [`Entity`], unlike [`IndexMap`](crate::index_map::IndexMap).
/// You'll need the assistance of [`EntityManager`](crate::entity::EntityManager) to create the key for indexing data.
pub struct Array<E: Entity, T> {
    pub(crate) ptr: SparseIndices<E>,
    pub(crate) data: Vec<T>,
    pub(crate) entities: Vec<E>,
}

impl<E: Entity, T> Default for Array<E, T> {
    fn default() -> Self {
        Self {
            ptr: SparseIndices::default(),
            data: Vec::default(),
            entities: Vec::default(),
        }
    }
}

impl<E: Entity, T: std::fmt::Debug> std::fmt::Debug for Array<E, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter())
            .finish()
    }
}

impl<E: Entity, T> Array<E, T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            ptr: SparseIndices::default(),
            data: Vec::with_capacity(capacity),
            entities: Vec::with_capacity(capacity),
        }
    }

    // pub(crate) fn get_raw(&self, entity: &E) -> Option<&UnsafeCell<T>> {
    //     self.ptr
    //         .get_index(entity)
    //         .map(|index| &self.data[index.index()])
    // }

    pub fn get(&self, entity: &E) -> Option<&T> {
        self.ptr
            .get_index(entity)
            .map(|index| &self.data[index.index()])
    }

    pub fn get_mut(&mut self, entity: &E) -> Option<&mut T> {
        self.ptr
            .get_index(entity)
            .map(|index| &mut self.data[index.index()])
    }

    pub fn get_or_insert(&mut self, entity: &E, value: impl FnOnce() -> T) -> &mut T {
        if let Some(index) = self.ptr.get_index(entity)
            && !index.is_null()
        {
            return &mut self.data[index.index()]
        }

        let data_index = self.data.len();

        self.ptr.set_index(entity, data_index);
        self.data.push(value());
        self.entities.push(*entity);

        self.get_mut(entity).unwrap()
    }

    /// Inserting or replacing the value
    pub fn insert(&mut self, entity: &E, value: T) {
        if let Some(index) = self.ptr.get_index(entity)
            && !index.is_null()
        {
            self.data[index.index()] = value;
            return;
        }

        let data_index = self.data.len();

        self.ptr.set_index(entity, data_index);
        self.data.push(value);
        self.entities.push(*entity);
    }

    /// The contiguousness of the data is guaranteed after removal via [`Vec::swap_remove`],
    /// but the order of the data is is not.
    pub fn remove(&mut self, entity: E) -> Option<T> {
        if self.data.is_empty() { return None }

        self.ptr
            .get_index(&entity)
            .cloned()
            .map(|idx| {
                let last = self.entities.last().unwrap();

                self.ptr.set_index(last, idx.index());
                self.ptr.set_null(&entity);
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

    pub fn contains(&self, entity: &E) -> bool {
        self.entities.contains(entity)
    }

    pub fn shrink_to_fit(&mut self) {
        self.ptr.shrink_to_fit();
        self.data.shrink_to_fit();
    }

    pub fn reset(&mut self) {
        self.ptr.reset();
        self.data.clear();
    }

    pub fn entity_data_index(&self, entity: &E) -> Option<usize> {
        self.ptr.get_index(entity).map(|i| i.index())
    }

    pub fn iter(&self) -> ArrayIter<'_, E, T> {
        ArrayIter::new(self)
    }

    pub fn iter_mut(&mut self) -> ArrayIterMut<'_, E, T> {
        ArrayIterMut::new(self)
    }

    pub fn iter_data_index(&self) -> impl Iterator<Item = usize> {
        self.ptr.iter_data_index()
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

pub struct ArrayIter<'a, E: Entity, T> {
    inner: Zip<Iter<'a, E>, Iter<'a, T>>,
}

impl<'a, E: Entity, T> ArrayIter<'a, E, T> {
    pub(crate) fn new(ds: &'a Array<E, T>) -> Self {
        let inner = ds.entities
            .iter()
            .zip(ds.data.iter());
        Self {
            inner,
        }
    }
}

impl<'a, E: Entity, T> Iterator for ArrayIter<'a, E, T> {
    type Item = (&'a E, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub struct ArrayIterMut<'a, E: Entity, T> {
    inner: Zip<Iter<'a, E>, IterMut<'a, T>>,
}

impl<'a, E: Entity, T> ArrayIterMut<'a, E, T> {
    pub(crate) fn new(ds: &'a mut Array<E, T>) -> Self {
        let inner = ds.entities
            .iter()
            .zip(ds.data.iter_mut());
        Self {
            inner,
        }
    }
}

impl<'a, E: Entity, T> Iterator for ArrayIterMut<'a, E, T> {
    type Item = (&'a E, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
