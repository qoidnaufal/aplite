use std::iter::{Enumerate, Filter, FilterMap, Zip};
use std::slice::{Iter, IterMut};

use crate::entity::Entity;
use super::sparse_index::SparseIndex;

/// A dense data storage which is guaranteed even after removal.
/// Doesn't facilitate the creation of [`Entity`], unlike [`IndexMap`](crate::index_map::IndexMap).
/// You'll need the assistance of [`EntityManager`](crate::entity::EntityManager) to create the key for indexing data.
pub struct DenseColumn<E: Entity, T> {
    pub(crate) ptr: SparseIndex<E>,
    pub(crate) data: Vec<T>,
}

impl<E: Entity, T> Default for DenseColumn<E, T> {
    fn default() -> Self {
        Self {
            ptr: SparseIndex::default(),
            data: Vec::default(),
        }
    }
}

impl<E: Entity, T: std::fmt::Debug> std::fmt::Debug for DenseColumn<E, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter())
            .finish()
    }
}

impl<E: Entity, T> DenseColumn<E, T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            ptr: SparseIndex::default(),
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    pub fn get(&self, entity: &E) -> Option<&T> {
        self.ptr
            .with(entity, |index| &self.data[index])
    }

    pub fn get_mut(&mut self, entity: &E) -> Option<&mut T> {
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

    pub fn iter(&self) -> DenseColumnIter<'_, T> {
        DenseColumnIter::new(self)
    }

    pub fn iter_mut(&mut self) -> DenseColumnIterMut<'_, T> {
        DenseColumnIterMut::new(self)
    }

    pub fn iter_data_index(&self) -> impl Iterator<Item = &usize> {
        self.ptr.iter_data_index()
    }

    pub fn iter_map(&self) -> MappedDenseColumnIter<'_, E, T> {
        MappedDenseColumnIter::new(self)
    }
}

/*
#########################################################
#                                                       #
#                        Iterator                       #
#                                                       #
#########################################################
*/

pub struct DenseColumnIter<'a, T> {
    inner: ZippedFilter<'a, T>,
}

fn filter(idx: &&usize) -> bool {
    idx != &&usize::MAX
}

type FnFilter = fn(&&usize) -> bool;
type ZippedFilter<'a, T> = Zip<Filter<Iter<'a, usize>, FnFilter>, Iter<'a, T>>;

impl<'a, T> DenseColumnIter<'a, T> {
    pub(crate) fn new<E: Entity>(ds: &'a DenseColumn<E, T>) -> Self {
        let inner = ds.ptr.ptr
            .iter()
            .filter(filter as FnFilter)
            .zip(ds.data.iter());
        Self {
            inner,
        }
    }
}

impl<'a, T> Iterator for DenseColumnIter<'a, T> {
    type Item = (&'a usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub struct MappedDenseColumnIter<'a, E: Entity, T> {
    inner: MappedZipFilter<'a, E, T>,
}

fn filter_map<E: Entity>((i, idx): (usize, &usize)) -> Option<E> {
    (idx != &usize::MAX).then_some(E::new(i as u32, 0))
}

type FnFilterMap<E> = fn((usize, &usize)) -> Option<E>;
type MappedZipFilter<'a, E, T> = Zip<FilterMap<Enumerate<Iter<'a, usize>>, FnFilterMap<E>>, Iter<'a, T>>;

impl<'a, E: Entity, T> MappedDenseColumnIter<'a, E, T> {
    pub(crate) fn new(ds: &'a DenseColumn<E, T>) -> Self {
        let inner = ds.ptr.ptr
            .iter()
            .enumerate()
            .filter_map(filter_map as FnFilterMap<E>)
            .zip(ds.data.iter());
        Self {
            inner,
        }
    }
}

impl<'a, E: Entity, T> Iterator for MappedDenseColumnIter<'a, E, T> {
    type Item = (E, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub struct DenseColumnIterMut<'a, T> {
    inner: ZippedFilterMut<'a, T>,
}

type ZippedFilterMut<'a, T> = Zip<Filter<Iter<'a, usize>, FnFilter>, IterMut<'a, T>>;

impl<'a, T> DenseColumnIterMut<'a, T> {
    pub(crate) fn new<E: Entity>(ds: &'a mut DenseColumn<E, T>) -> Self {
        let inner = ds.ptr.ptr
            .iter()
            .filter(filter as FnFilter)
            .zip(ds.data.iter_mut());
        Self {
            inner,
        }
    }
}

impl<'a, T> Iterator for DenseColumnIterMut<'a, T> {
    type Item = (&'a usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
