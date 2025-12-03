use std::cell::UnsafeCell;
use std::iter::Zip;
use std::slice::{Iter, IterMut};

use crate::entity::EntityId;
use crate::sparse_set::indices::SparseIndices;

/// A typed SparseSet backed by [`Vec`]
pub struct SparseSet<T> {
    pub(crate) data: Vec<UnsafeCell<T>>,
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
            .map(|index| unsafe {
                &*self.data[index].get()
            })
    }

    pub fn get_mut(&mut self, id: &EntityId) -> Option<&mut T> {
        self.indexes
            .get_index(id)
            .map(|index| {
                self.data[index].get_mut()
            })
    }

    /// Inserting or replacing the value
    pub fn insert(&mut self, id: &EntityId, value: T) {
        if let Some(index) = self.indexes.get_index(id) {
            *self.data[index].get_mut() = value;
            return;
        }

        let data_index = self.data.len();

        self.indexes.set_index(id, data_index);
        self.data.push(UnsafeCell::new(value));
        self.entities.push(*id);
    }

    /// The contiguousness of the data is guaranteed after removal via [`Vec::swap_remove`],
    /// but the order of the data is is not.
    pub fn remove(&mut self, id: EntityId) -> Option<T> {
        self.indexes
            .get_index(&id)
            .map(|index| {
                let last = self.entities.last().unwrap();

                self.indexes.set_index(last, index);
                self.indexes.set_null(&id);
                self.entities.swap_remove(index);
                self.data.swap_remove(index).into_inner()
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

    pub fn reset(&mut self) {
        self.indexes.reset();
        self.data.clear();
    }

    pub fn entity_data_index(&self, id: &EntityId) -> Option<usize> {
        self.indexes.get_index(id)
    }

    pub fn iter(&self) -> SparseSetIter<'_, T> {
        SparseSetIter::new(self)
    }

    pub fn iter_mut(&mut self) -> SparseSetIterMut<'_, T> {
        SparseSetIterMut::new(self)
    }

    pub fn iter_data_index(&self) -> impl Iterator<Item = usize> {
        self.indexes.iter_data_index()
    }
}

/*
#########################################################
#                                                       #
#                    Array::Iterator                    #
#                                                       #
#########################################################
*/

pub struct SparseSetIter<'a, T> {
    inner: Zip<Iter<'a, EntityId>, Iter<'a, UnsafeCell<T>>>,
}

impl<'a, T> SparseSetIter<'a, T> {
    pub(crate) fn new(ds: &'a SparseSet<T>) -> Self {
        let inner = ds.entities
            .iter()
            .zip(ds.data.iter());
        Self {
            inner,
        }
    }
}

impl<'a, T> Iterator for SparseSetIter<'a, T> {
    type Item = (&'a EntityId, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(id, cell)| unsafe { (id, &*cell.get()) })
    }
}

pub struct SparseSetIterMut<'a, T> {
    inner: Zip<Iter<'a, EntityId>, IterMut<'a, UnsafeCell<T>>>,
}

impl<'a, T> SparseSetIterMut<'a, T> {
    pub(crate) fn new(ds: &'a mut SparseSet<T>) -> Self {
        let inner = ds.entities
            .iter()
            .zip(ds.data.iter_mut());
        Self {
            inner,
        }
    }
}

impl<'a, T> Iterator for SparseSetIterMut<'a, T> {
    type Item = (&'a EntityId, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(id, cell)| (id, cell.get_mut()))
    }
}
