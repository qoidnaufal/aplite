use std::marker::PhantomData;
use crate::entity::Entity;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Index(usize);

impl Index {
    pub(crate) fn new(data_index: usize) -> Self {
        Self(data_index)
    }

    pub(crate) fn index(&self) -> usize {
        self.0
    }

    pub(crate) fn null() -> Self {
        Self(usize::MAX)
    }

    pub(crate) fn is_null(&self) -> bool {
        self.0 == usize::MAX
    }
}

pub struct SparseIndices<E: Entity> {
    pub(crate) ptr: Vec<Index>,
    marker: PhantomData<E>,
}

impl<E: Entity> Default for SparseIndices<E> {
    fn default() -> Self {
        Self {
            ptr: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<E: Entity> SparseIndices<E> {
    pub(crate) fn get_index(&self, entity: &E) -> Option<&Index> {
        self.ptr.get(entity.index()).filter(|i| !i.is_null())
    }

    pub fn get_data_index(&self, entity: &E) -> Option<usize> {
        self.ptr
            .get(entity.index())
            .and_then(|i| (!i.is_null()).then_some(i.index()))
    }

    pub fn set_index(&mut self, index: usize, data_index: usize) {
        if index >= self.ptr.len() {
            self.resize(index);
        }
        self.ptr[index] = Index::new(data_index);
    }

    pub fn set_null(&mut self, entity: &E) {
        self.ptr[entity.index()] = Index::null()
    }

    pub fn with<F, T>(&self, entity: &E, f: F) -> Option<T>
    where
        F: FnOnce(usize) -> T
    {
        self.get_index(entity).map(|index| f(index.index()))
    }

    pub fn contains(&self, entity: &E) -> bool {
        self.get_index(entity).is_some()
    }

    pub fn resize_if_needed(&mut self, entity: &E) {
        let index = entity.index();
        if index >= self.ptr.len() {
            self.resize(index);
        }
    }

    pub(crate) fn resize(&mut self, new_len: usize) {
        self.ptr.resize(new_len + 1, Index::null());
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.ptr.shrink_to_fit();
    }

    pub fn len(&self) -> usize {
        self.ptr.len()
    }

    pub fn reset(&mut self) {
        self.ptr.clear();
        self.ptr.shrink_to_fit();
    }

    /// Iterate over the index of the associated entity
    pub fn iter_entity_index(&self) -> impl Iterator<Item = usize> {
        self.ptr
            .iter()
            .enumerate()
            .filter_map(|(i, idx)| (!idx.is_null()).then_some(i))
    }

    /// Iterate over the position of the indexed data
    pub fn iter_data_index(&self) -> impl Iterator<Item = usize> {
        self.ptr
            .iter()
            .filter_map(|i| (!i.is_null()).then_some(i.index()))
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
    use crate::data::array::Array;
    use crate::data::table::Table;
    use crate::data::component::QueryOne;
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
        let mut store = Array::<TestId, String>::with_capacity(NUM);

        for i in 0..NUM {
            store.insert(&ids[i], (i + 1).to_string());
        }

        let exist = store.get(&TestId(5));
        assert!(exist.is_some());
        assert_eq!(exist.unwrap(), "5");
    }

    #[test]
    fn remove() {
        const NUM: usize = 10;
        let ids = setup_entity(NUM);
        let mut store = Array::<TestId, ()>::with_capacity(NUM);

        for i in 0..NUM {
            store.insert(&ids[NUM - 1 - i], ());
        }

        let index = &ids[0];
        let index_before_remove = store.entity_data_index(index);
        assert!(index_before_remove.is_some());

        let removed = store.remove(*index);
        assert!(removed.is_some());

        let index_after_remove = store.entity_data_index(index);
        assert!(index_after_remove.is_none());
    }

    #[test]
    fn iter() {
        const NUM: usize = 10;
        let ids = setup_entity(NUM);
        let mut store = Array::<TestId, String>::with_capacity(NUM);

        for i in 0..NUM {
            store.insert(&ids[NUM - 1 - i], i.to_string());
        }

        let count = store.iter().count();
        assert_eq!(count, store.len());
    }

    #[test]
    fn multi_store_query() {
        const NUM: usize = 10;
        let mut ms = Table::with_capacity(NUM);
        let ids = setup_entity(NUM);
        for i in 0..NUM {
            let id = &ids[i];
            ms.insert_one(id, i);
            ms.insert_one(id, format!("{id:?}"));
        }
        let query_usize = QueryOne::<usize>::new(&ms);
        let query_string = QueryOne::<String>::new(&ms);
        assert_eq!(query_string.into_iter().count(), query_usize.into_iter().count());
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
