use std::iter::{Enumerate, Filter, FilterMap, Zip};
use std::marker::PhantomData;
use std::slice::{Iter, IterMut};

use crate::entity::Entity;
use super::sparse_index::{SparseIndex, Index};

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
            .get_index(entity)
            .map(|index| &self.data[index])
    }

    pub fn get_mut(&mut self, entity: &E) -> Option<&mut T> {
        self.ptr
            .get_index(entity)
            .map(|index| &mut self.data[index])
    }

    /// Inserting or replacing the value
    pub fn insert(&mut self, entity: &E, value: T) {
        if let Some(index) = self.ptr.get_index(entity)
            && index != usize::MAX
        {
            self.data[index] = value;
            return;
        }

        let entity_index = entity.index();

        if entity_index >= self.ptr.len() {
            self.ptr.resize(entity_index);
        }

        let data_index = self.data.len();
        self.data.push(value);
        self.ptr.set_index_from_entity(entity, data_index);
    }

    /// The contiguousness of the data is guaranteed after removal via [`Vec::swap_remove`],
    /// but the order of the data is is not.
    pub fn remove(&mut self, entity: E) -> Option<T> {
        if self.data.is_empty() { return None }

        self.ptr.get_index(&entity)
            .and_then(|idx| {
                let swap_index = self.data.len() - 1;

                let pos = self.ptr
                    .iter_all()
                    .position(|i| i == swap_index);

                pos.map(|pos| {
                    self.ptr.set_index_from_usize(pos, idx);
                    self.ptr.set_null(&entity);
                    self.data.swap_remove(idx)
                })
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
        self.ptr.contains(entity)
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
        self.ptr.get_index(entity)
    }

    pub fn iter(&self) -> DenseColumnIter<'_, E, T> {
        DenseColumnIter::new(self)
    }

    pub fn iter_mut(&mut self) -> DenseColumnIterMut<'_, E, T> {
        DenseColumnIterMut::new(self)
    }

    pub fn iter_data_index(&self) -> impl Iterator<Item = usize> {
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

pub struct DenseColumnIter<'a, E: Entity, T> {
    inner: ZippedFilter<'a, T>,
    marker: PhantomData<E>,
}

fn filter(idx: &&Index) -> bool {
    !idx.is_null()
}

type FnFilter = fn(&&Index) -> bool;
type ZippedFilter<'a, T> = Zip<Filter<Iter<'a, super::sparse_index::Index>, FnFilter>, Iter<'a, T>>;

impl<'a, E: Entity, T> DenseColumnIter<'a, E, T> {
    pub(crate) fn new(ds: &'a DenseColumn<E, T>) -> Self {
        let inner = ds.ptr.ptr
            .iter()
            .filter(filter as FnFilter)
            .zip(ds.data.iter());
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<'a, E: Entity, T> Iterator for DenseColumnIter<'a, E, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(i, t)| (i.index::<E>(), t))
    }
}

pub struct MappedDenseColumnIter<'a, E: Entity, T> {
    inner: MappedZipFilter<'a, E, T>,
}

fn filter_map<E: Entity>((i, idx): (usize, &Index)) -> Option<E> {
    (!idx.is_null()).then_some(E::new(i as u32, idx.version::<E>()))
}

type FnFilterMap<E> = fn((usize, &Index)) -> Option<E>;
type MappedZipFilter<'a, E, T> = Zip<FilterMap<Enumerate<Iter<'a, Index>>, FnFilterMap<E>>, Iter<'a, T>>;

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

pub struct DenseColumnIterMut<'a, E: Entity, T> {
    inner: ZippedFilterMut<'a, T>,
    marker: PhantomData<E>,
}

type ZippedFilterMut<'a, T> = Zip<Filter<Iter<'a, Index>, FnFilter>, IterMut<'a, T>>;

impl<'a, E: Entity, T> DenseColumnIterMut<'a, E, T> {
    pub(crate) fn new(ds: &'a mut DenseColumn<E, T>) -> Self {
        let inner = ds.ptr.ptr
            .iter()
            .filter(filter as FnFilter)
            .zip(ds.data.iter_mut());
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<'a, E: Entity, T> Iterator for DenseColumnIterMut<'a, E, T> {
    type Item = (usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(i, t)| (i.index::<E>(), t))
    }
}
