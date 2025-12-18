use std::cell::UnsafeCell;
use std::iter::Zip;
use std::slice::{Iter, IterMut};

use crate::entity::EntityId;
use crate::sparse_set::indices::SparseIndices;

/// A typed SparseSet backed by [`Vec`]
pub struct SparseSet<T> {
    pub(crate) data: Vec<T>,
    pub(crate) indexes: SparseIndices,
    pub(crate) entities: Vec<EntityId>,
}

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self::new()
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
    pub const fn new() -> Self {
        Self {
            data: Vec::new(),
            indexes: SparseIndices::new(),
            entities: Vec::new(),
        }
    }

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

    pub fn get(&self, id: EntityId) -> Option<&T> {
        self.indexes
            .get_index(id)
            .map(|index| &self.data[index])
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        self.indexes
            .get_index(id)
            .map(|index| &mut self.data[index])
    }

    pub fn get_cell(&self, entity: EntityId) -> Option<&UnsafeCell<T>> {
        self.indexes
            .get_index(entity)
            .map(|index| unsafe {
                let raw = self.data.as_ptr();
                let cell = raw.add(index).cast::<UnsafeCell<T>>();
                &*cell
            })
    }

    pub fn get_raw(&mut self, entity: EntityId) -> Option<*mut T> {
        self.indexes
            .get_index(entity)
            .map(|index| &mut self.data[index] as *mut T)
    }

    /// Inserting or replacing the value
    pub fn insert(&mut self, id: EntityId, value: T) {
        if let Some(index) = self.indexes.get_index(id) {
            self.data[index] = value;
            return;
        }

        let data_index = self.data.len();

        self.indexes.set_index(id, data_index);
        self.data.push(value);
        self.entities.push(id);
    }

    /// The contiguousness of the data is guaranteed after removal via [`Vec::swap_remove`],
    /// but the order of the data is is not.
    pub fn remove(&mut self, id: EntityId) -> Option<T> {
        self.indexes
            .get_index(id)
            .map(|index| {
                let last = self.entities.last().unwrap();

                self.indexes.set_index(*last, index);
                self.indexes.set_null(id);
                self.entities.swap_remove(index);
                self.data.swap_remove(index)
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

    pub fn entity_data_index(&self, id: EntityId) -> Option<usize> {
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
    inner: Zip<Iter<'a, EntityId>, Iter<'a, T>>,
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
        self.inner.next()
    }
}

pub struct SparseSetIterMut<'a, T> {
    inner: Zip<Iter<'a, EntityId>, IterMut<'a, T>>,
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
        self.inner.next()
    }
}

#[cfg(test)]
mod typed_sparse_set {
    use super::*;

    struct Obj {
        name: String,
        age: u32,
    }

    impl std::fmt::Debug for Obj {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Obj")
                .field("name", &self.name)
                .field("age", &self.age)
                .finish()
        }
    }

    impl From<u32> for Obj {
        fn from(value: u32) -> Self {
            Self {
                name: value.to_string(),
                age: value,
            }
        }
    }

    #[test]
    fn cell_test() {
        let mut set = SparseSet::<Obj>::with_capacity(5);
        for i in 0..5 {
            let id = EntityId::new(i);
            set.insert(id, i.into());
        }

        let cell_1 = set.get_cell(EntityId::new(1));
        assert!(cell_1.is_some());
        unsafe {
            println!("{:?}", &*cell_1.unwrap().get())
        }
    }
}
