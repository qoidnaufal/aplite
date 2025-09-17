use crate::entity::Entity;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Index(usize);

impl Index {
    pub(crate) fn new<E: Entity>(entity: &E, data_index: usize) -> Self {
        Self((entity.version() << E::VERSION_BITS) as usize | data_index)
    }

    pub(crate) fn null() -> Self {
        Self(usize::MAX)
    }

    pub(crate) fn index<E: Entity>(&self) -> usize {
        self.0 & E::INDEX_MASK as usize
    }

    pub(crate) fn version<E: Entity>(&self) -> u16 {
        (self.0 >> E::INDEX_BITS) as u16 & E::VERSION_MASK
    }

    pub(crate) fn is_null(&self) -> bool {
        self.0 == usize::MAX
    }
}

pub struct SparseIndex<E: Entity> {
    pub(crate) ptr: Vec<Index>,
    marker: std::marker::PhantomData<E>,
}

impl<E: Entity> Default for SparseIndex<E> {
    fn default() -> Self {
        Self {
            ptr: Vec::new(),
            marker: std::marker::PhantomData,
        }
    }
}

impl<E: Entity> SparseIndex<E> {
    pub fn get_index(&self, entity: &E) -> Option<usize> {
        self.ptr.get(entity.index())
            .and_then(|i| (!i.is_null()).then_some(i.index::<E>()))
    }

    pub(crate) fn set_index_from_entity(&mut self, entity: &E, data_index: usize) {
        self.ptr[entity.index()] = Index::new(entity, data_index)
    }

    pub(crate) fn set_null(&mut self, entity: &E) {
        self.ptr[entity.index()] = Index::null()
    }

    pub(crate) fn set_index_from_usize(&mut self, index: usize, data_index: usize) {
        let version = self.ptr[index].version::<E>();
        let entity = E::new(index as u32, version);
        self.ptr[index] = Index::new(&entity, data_index);
    }

    pub fn with<F, T>(&self, entity: &E, f: F) -> Option<T>
    where
        F: FnOnce(usize) -> T
    {
        self.get_index(entity).map(f)
    }

    pub fn contains(&self, entity: &E) -> bool {
        self.get_index(entity).is_some()
    }

    pub(crate) fn resize(&mut self, new_len: usize) {
        self.ptr.resize(new_len + 4, Index::null());
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.ptr.shrink_to_fit();
    }

    pub(crate) fn len(&self) -> usize {
        self.ptr.len()
    }

    pub fn reset(&mut self) {
        self.ptr.clear();
        self.ptr.shrink_to_fit();
    }

    // pub fn position(&self, index: usize) {
    //     self.ptr
    // }

    pub fn iter_all(&self) -> impl Iterator<Item = usize> {
        self.ptr.iter().map(|i| i.index::<E>())
    }

    pub fn iter_data_index(&self) -> impl Iterator<Item = usize> {
        self.ptr
            .iter()
            .filter_map(|i| (!i.is_null()).then_some(i.index::<E>()))
    }

    pub fn iter_entity_index(&self) -> impl Iterator<Item = E> {
        self.ptr
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                (!p.is_null()).then_some({
                    E::new(i as u32, p.version::<E>())
                })
            })
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
    use crate::data::dense_column::DenseColumn;
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
        let mut store = DenseColumn::<TestId, String>::with_capacity(NUM);

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
        let mut store = DenseColumn::<TestId, ()>::with_capacity(NUM);

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
        let mut store = DenseColumn::<TestId, String>::with_capacity(NUM);

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
        let query_usize = QueryOne::<TestId, usize>::new(&ms);
        let query_string = QueryOne::<TestId, String>::new(&ms);
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
