use std::iter::{Zip, Map, FilterMap, Enumerate};
use std::marker::PhantomData;
use std::slice::{Iter, IterMut};
use std::cell::UnsafeCell;

use crate::entity::Entity;
use super::sparse_index::{SparseIndices, Index};
use super::component::QueryData;
use super::component::{map_query, FnMapQuery};

/// A dense data storage which is guaranteed even after removal.
/// Doesn't facilitate the creation of [`Entity`], unlike [`IndexMap`](crate::index_map::IndexMap).
/// You'll need the assistance of [`EntityManager`](crate::entity::EntityManager) to create the key for indexing data.
pub struct Array<E: Entity, T> {
    pub(crate) ptr: SparseIndices<E>,
    pub(crate) data: Vec<UnsafeCell<T>>,
    pub(crate) entities: Vec<E>,
}

/// A dense data storage which doesn't support insertion or removal.
/// Doesn't facilitate the creation of [`Entity`], unlike [`IndexMap`](crate::index_map::IndexMap).
/// You'll need the assistance of [`EntityManager`](crate::entity::EntityManager) to create the key for indexing data.
pub struct ImmutableArray<E: Entity, T> {
    ptr: Box<[Index]>,
    data: Box<[UnsafeCell<T>]>,
    marker: PhantomData<E>,
}

impl<E: Entity, T> From<Array<E, T>> for ImmutableArray<E, T> {
    fn from(arr: Array<E, T>) -> Self {
        Self {
            ptr: arr.ptr.ptr.into_boxed_slice(),
            data: arr.data.into_boxed_slice(),
            marker: std::marker::PhantomData,
        }
    }
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

    pub(crate) fn get_raw(&self, entity: &E) -> Option<&UnsafeCell<T>> {
        self.ptr
            .get_index(entity)
            .map(|index| &self.data[index.index()])
    }

    pub fn get(&self, entity: &E) -> Option<&T> {
        self.ptr
            .get_index(entity)
            .map(|index| unsafe {
                &*self.data[index.index()].get()
            })
    }

    pub fn get_mut(&mut self, entity: &E) -> Option<&mut T> {
        self.ptr
            .get_index(entity)
            .map(|index| {
                self.data[index.index()].get_mut()
            })
    }

    /// Inserting or replacing the value
    pub fn insert(&mut self, entity: &E, value: T) {
        if let Some(index) = self.ptr.get_index(entity)
            && !index.is_null()
        {
            *self.data[index.index()].get_mut() = value;
            return;
        }

        let entity_index = entity.index();

        if entity_index >= self.ptr.len() {
            self.ptr.resize(entity_index);
        }

        let data_index = self.data.len();
        self.data.push(UnsafeCell::new(value));
        self.entities.push(*entity);
        self.ptr.set_index(entity_index, data_index);
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

                self.ptr.set_index(last.index(), idx.index());
                self.ptr.set_null(&entity);
                self.entities.swap_remove(idx.index());
                self.data.swap_remove(idx.index()).into_inner()
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

impl<E: Entity, T: 'static> Array<E, T> {
    pub(crate) fn query_one<'a, Q>(&'a self) -> Map<Iter<'a, UnsafeCell<T>>, FnMapQuery<'a, Q>>
    where
        Q: QueryData<'a, Item = T>,
    {
        self.data.iter().map(map_query::<'a, Q> as FnMapQuery<'a, Q>)
    }
}

impl<E: Entity, T> ImmutableArray<E, T> {
    pub fn get(&self, entity: &E) -> Option<&T> {
        self.ptr.get(entity.index())
            .map(|index| unsafe {
                &*self.data[index.index()].get()
            })
    }

    pub fn get_mut(&mut self, entity: &E) -> Option<&mut T> {
        self.ptr.get(entity.index())
            .map(|index| {
                self.data[index.index()].get_mut()
            })
    }

    pub fn contains(&self, entity: &E) -> bool {
        self.ptr.get(entity.index())
            .is_some_and(|index| !index.is_null())
    }

    pub fn iter(&self) -> ImmutableArrayIter<'_, E, T> {
        ImmutableArrayIter {
            inner: self.ptr.iter()
                .enumerate()
                .filter_map(filter_map as fn((usize, &Index)) -> Option<usize>)
                .zip(self.data.as_ref()),
            marker: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> ImmutableArrayIterMut<'_, E, T> {
        ImmutableArrayIterMut {
            inner: self.ptr.iter()
                .enumerate()
                .filter_map(filter_map as fn((usize, &Index)) -> Option<usize>)
                .zip(self.data.as_mut()),
            marker: PhantomData,
        }
    }
}

fn filter_map((i, index): (usize, &Index)) -> Option<usize> {
    (!index.is_null()).then_some(i)
}

/*
#########################################################
#                                                       #
#                    Array::Iterator                    #
#                                                       #
#########################################################
*/

pub struct ArrayIter<'a, E: Entity, T> {
    inner: Zip<Iter<'a, E>, Iter<'a, UnsafeCell<T>>>,
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
        self.inner
            .next()
            .map(|(i, cell)| unsafe {
                (i, &*cell.get())
            })
    }
}

pub struct ArrayIterMut<'a, E: Entity, T> {
    inner: Zip<Iter<'a, E>, IterMut<'a, UnsafeCell<T>>>,
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
        self.inner
            .next()
            .map(|(i, cell)| {
                (i, cell.get_mut())
            })
    }
}

/*
#########################################################
#                                                       #
#               ImmutableArray::Iterator                #
#                                                       #
#########################################################
*/

pub struct ImmutableArrayIter<'a, E: Entity, T> {
    inner: Zip<FilterMap<Enumerate<Iter<'a, Index>>, fn((usize, &Index)) -> Option<usize>>, Iter<'a, UnsafeCell<T>>>,
    marker: PhantomData<E>,
}

impl<'a, E: Entity, T> Iterator for ImmutableArrayIter<'a, E, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(i, cell)| unsafe {
                (i, &*cell.get())
            })
    }
}

pub struct ImmutableArrayIterMut<'a, E: Entity, T> {
    inner: Zip<FilterMap<Enumerate<Iter<'a, Index>>, fn((usize, &Index)) -> Option<usize>>, IterMut<'a, UnsafeCell<T>>>,
    marker: PhantomData<E>,
}

impl<'a, E: Entity, T> Iterator for ImmutableArrayIterMut<'a, E, T> {
    type Item = (usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(i, cell)| {
                (i, cell.get_mut())
            })
    }
}
