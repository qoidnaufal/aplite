use crate::entity::Entity;
use crate::iterator::{
    MappedDataStoreIter,
    DataStoreIter,
    DataStoreIterMut
};

/// A Contiguous data storage which is guaranteed even after removal.
/// Doesn't facilitate the creation of [`Entity`], unlike [`IndexMap`](crate::index_map::IndexMap).
/// You'll need the assistance of [`EntityManager`](crate::entity::EntityManager) to create the key for indexing data.
#[derive(Default)]
pub struct DataStore<E: Entity, T> {
    pub(crate) ptr: DataPointer<E>,
    pub(crate) data: Vec<T>,
}

impl<E: crate::Entity, T: std::fmt::Debug> std::fmt::Debug for DataStore<E, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter())
            .finish()
    }
}

impl<E: Entity, T> DataStore<E, T> {
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

// pub struct DataTable<E: Entity, T> {
//     ptr: DataPointer<E>,
//     rows: std::collections::HashMap<std::any::TypeId, Vec<T>>,
// }

/*
#########################################################
#                                                       #
#                      DataPointer                      #
#                                                       #
#########################################################
*/

pub struct DataPointer<E: Entity> {
    pub(crate) ptr: Vec<usize>,
    marker: std::marker::PhantomData<E>,
}

impl<E: Entity> Default for DataPointer<E> {
    fn default() -> Self {
        Self {
            ptr: Vec::new(),
            marker: std::marker::PhantomData,
        }
    }
}

impl<E: Entity> DataPointer<E> {
    pub fn get<'a, T>(&self, entity: E, data: &'a [T]) -> Option<&'a T> {
        let entity_index = entity.index();
        self.ptr
            .get(entity_index)
            .and_then(|&idx| {
                (idx != usize::MAX)
                    .then_some(&data[idx])
            })
    }

    pub fn get_mut<'a, T>(&self, entity: E, data: &'a mut [T]) -> Option<&'a mut T> {
        let entity_index = entity.index();
        self.ptr
            .get(entity_index)
            .and_then(|&idx| {
                (idx != usize::MAX)
                    .then_some(&mut data[idx])
            })
    }

    pub fn with<F, T>(&self, entity: E, f: F) -> Option<T>
    where
        F: FnOnce(usize) -> T
    {
        let entity_index = entity.index();
        self.ptr
            .get(entity_index)
            .and_then(|&idx| {
                (idx != usize::MAX).then_some(f(idx))
            })
    }

    pub fn insert<T>(&mut self, entity: E, value: T, data: &mut Vec<T>) {
        let entity_index = entity.index();

        if let Some(index) = self.ptr.get(entity_index)
            && index != &usize::MAX
        {
            data[*index] = value;
            return;
        }

        if entity_index >= self.ptr.len() {
            self.ptr.resize(entity_index + 1, usize::MAX);
        }

        let data_index = data.len();
        data.push(value);
        self.ptr[entity_index] = data_index;
    }

    pub fn remove<T>(&mut self, entity: E, data: &mut Vec<T>) -> Option<T> {
        if data.is_empty() { return None }

        let entity_index = entity.index();

        if let Some(idx) = self.ptr.get(entity_index)
            && idx != &usize::MAX
        {
            let swap_index = data.len() - 1;
            let data_index_to_remove = self.ptr[entity_index];

            // FIXME: maybe there's a better way than this?
            // also try to resize when a certain capacity-to-len ratio exceeded
            self.ptr
                .iter()
                .position(|i| *i == swap_index)
                .map(|pos| {
                    self.ptr[pos] = data_index_to_remove;
                    self.ptr[entity_index] = usize::MAX;
                    data.swap_remove(data_index_to_remove)
                })
        } else {
            None
        }
    }

    pub fn contains(&self, entity: E) -> bool {
        entity.index() <= self.ptr.len()
            && self.ptr[entity.index()] != usize::MAX
    }

    pub fn entity_data_index(&self, entity: E) -> Option<usize> {
        if entity.index() < self.ptr.len() {
            let index = self.ptr[entity.index()];
            (index != usize::MAX).then_some(index)
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.ptr.clear();
        self.ptr.shrink_to_fit();
    }

    pub fn iter_data_index(&self) -> impl Iterator<Item = &usize> {
        self.ptr
            .iter()
            .filter(|p| (p != &&usize::MAX))
    }

    pub fn iter_entity_index(&self) -> impl Iterator<Item = E> {
        self.ptr
            .iter()
            .enumerate()
            .filter_map(|(idx, p)| (p != &usize::MAX)
                .then_some({
                    E::new(idx as u32, 0)
                }))
    }
}

/*
#########################################################
#                                                       #
#                         TEST                          #
#                                                       #
#########################################################
*/

#[cfg(test)]
mod store_test {
    use super::DataStore;
    use crate::{EntityManager, Entity, create_entity};

    create_entity! { TestId }

    fn setup_entity(num: usize) -> Vec<TestId> {
        let mut manager = EntityManager::<TestId>::with_capacity(num);
        let mut vec = vec![];
        for _ in 0..num {
            let id = manager.create();
            vec.push(id);
        }

        vec
    }

    #[test]
    fn insert_get() {
        const NUM: usize = 10;
        let ids = setup_entity(NUM);
        let mut store = DataStore::<TestId, String>::with_capacity(NUM);

        for i in 0..NUM {
            store.insert(ids[i], (i + 1).to_string());
        }

        let exist = store.get(TestId(5));
        assert!(exist.is_some());
        assert_eq!(exist.unwrap(), "5");
    }

    #[test]
    fn remove() {
        const NUM: usize = 10;
        let ids = setup_entity(NUM);
        let mut store = DataStore::<TestId, ()>::with_capacity(NUM);

        for i in 0..NUM {
            store.insert(ids[NUM - 1 - i], ());
        }

        let index = ids[0];
        let index_before_remove = store.entity_data_index(index);
        assert!(index_before_remove.is_some());

        let removed = store.remove(index);
        assert!(removed.is_some());

        let index_after_remove = store.entity_data_index(index);
        assert!(index_after_remove.is_none());
    }

    #[test]
    fn iter() {
        const NUM: usize = 10;
        let ids = setup_entity(NUM);
        let mut store = DataStore::<TestId, String>::with_capacity(NUM);

        for i in 0..NUM {
            store.insert(ids[NUM - 1 - i], i.to_string());
        }

        let count = store.iter().count();
        assert_eq!(count, store.len());
        // eprintln!("{store:#?}");
    }

    // #[test]
    // fn stress_test() {
    //     const NUM: usize = 100_000;
    //     let ids = setup_entity(NUM);
    //     let mut store = DataStore::<TestId, ()>::with_capacity(NUM);

    //     let now = std::time::Instant::now();
    //     for i in 0..NUM {
    //         store.insert(ids[i], ());
    //     }
    //     eprintln!("time for {NUM} insertion: {:?}", now.elapsed());

    //     let now = std::time::Instant::now();
    //     for i in 0..NUM {
    //         let _ = store.get(ids[i]);
    //     }
    //     let elapsed = now.elapsed();
    //     eprintln!("time for {NUM} get: {:?}", elapsed);
    //     eprintln!("average time per get: {:?}", elapsed.div_f64(NUM as f64));

    //     let now = std::time::Instant::now();
    //     for i in 0..NUM {
    //         let _ = store.remove(ids[i]);
    //     }
    //     let elapsed = now.elapsed();
    //     eprintln!("time for {NUM} removal: {:?}", elapsed);
    //     eprintln!("average time per removal: {:?}", elapsed.div_f64(NUM as f64));
    // }
}
