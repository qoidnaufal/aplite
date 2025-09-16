use std::any::Any;

use crate::entity::Entity;

pub struct SparseIndex<E: Entity> {
    pub(crate) ptr: Vec<usize>,
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
    pub fn get_any_ref<'a, T: 'static>(&self, entity: &E, data: &'a [Box<dyn Any>]) -> Option<&'a T> {
        let entity_index = entity.index();
        self.ptr
            .get(entity_index)
            .and_then(|&idx| {
                (idx != usize::MAX)
                    .then_some(data[idx].downcast_ref()?)
            })
    }

    pub fn get_any_mut<'a, T: 'static>(&self, entity: &E, data: &'a mut [Box<dyn Any>]) -> Option<&'a mut T> {
        let entity_index = entity.index();
        self.ptr
            .get(entity_index)
            .and_then(|&idx| {
                (idx != usize::MAX)
                    .then_some(data[idx].downcast_mut()?)
            })
    }

    pub fn insert_any<T: Any + 'static>(&mut self, entity: E, value: T, data: &mut Vec<Box<dyn Any>>) {
        let entity_index = entity.index();

        if let Some(index) = self.ptr.get(entity_index)
            && index != &usize::MAX
        {
            data[*index] = Box::new(value);
            return;
        }

        if entity_index >= self.ptr.len() {
            self.resize(entity_index);
        }

        let data_index = data.len();
        data.push(Box::new(value));
        self.ptr[entity_index] = data_index;
    }

    pub fn remove_any<T: 'static>(&mut self, entity: E, data: &mut Vec<Box<dyn Any>>) -> Option<T> {
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
                .enumerate()
                .filter_map(|(i, id)| (*id < usize::MAX).then_some(i))
                .find(|i| *i == swap_index)
                .and_then(|pos| {
                    self.ptr[pos] = data_index_to_remove;
                    self.ptr[entity_index] = usize::MAX;
                    data.swap_remove(data_index_to_remove)
                        .downcast::<T>()
                        .ok()
                        .map(|b| *b)
                })
        } else {
            None
        }
    }
}

impl<E: Entity> SparseIndex<E> {
    pub fn get<'a, T>(&self, entity: &'a E, data: &'a [T]) -> Option<&'a T> {
        let entity_index = entity.index();
        self.ptr
            .get(entity_index)
            .and_then(|&idx| {
                (idx != usize::MAX)
                    .then_some(&data[idx])
            })
    }

    pub fn get_mut<'a, T>(&self, entity: &'a E, data: &'a mut [T]) -> Option<&'a mut T> {
        let entity_index = entity.index();
        self.ptr
            .get(entity_index)
            .and_then(|&idx| {
                (idx != usize::MAX)
                    .then_some(&mut data[idx])
            })
    }

    pub fn with<F, T>(&self, entity: &E, f: F) -> Option<T>
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

    pub fn insert<T>(&mut self, entity: &E, value: T, data: &mut Vec<T>) {
        let entity_index = entity.index();

        if let Some(index) = self.ptr.get(entity_index)
            && index != &usize::MAX
        {
            data[*index] = value;
            return;
        }

        if entity_index >= self.ptr.len() {
            self.resize(entity_index);
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

    pub fn contains(&self, entity: &E) -> bool {
        entity.index() <= self.ptr.len()
            && self.ptr[entity.index()] != usize::MAX
    }

    pub fn entity_data_index(&self, entity: &E) -> Option<usize> {
        if entity.index() < self.ptr.len() {
            let index = self.ptr[entity.index()];
            (index != usize::MAX).then_some(index)
        } else {
            None
        }
    }

    fn resize(&mut self, new_len: usize) {
        self.ptr.resize(new_len + 4, usize::MAX);
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
