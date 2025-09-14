use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

use super::dense_row::DenseRow;
use super::query::{Query, FetchData, QueryData};

use crate::entity::Entity;

pub struct Table<E: Entity> {
    pub(crate) inner: HashMap<TypeId, DenseRow<E, Box<dyn Any>>>,
}

impl<E: Entity> Default for Table<E> {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<E: Entity> Table<E> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity)
        }
    }

    pub fn insert<T: Any + 'static>(&mut self, entity: E, value: T) {
        let type_id = TypeId::of::<T>();
        let data = self.inner.entry(type_id).or_default();
        data.insert(entity, Box::new(value));
    }

    pub fn get<T: 'static>(&self, entity: E) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.inner.get(&type_id)
            .and_then(|ds| {
                ds.ptr.get_any_ref(entity, &ds.data)
            })
    }

    pub fn get_mut<T: 'static>(&mut self, entity: E) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.inner.get_mut(&type_id)
            .and_then(|ds| {
                ds.ptr.get_any_mut(entity, &mut ds.data)
            })
    }

    pub fn query<T: 'static>(&self) -> Query<'_, E, T> {
        let type_id = TypeId::of::<T>();
        Query {
            inner: self.inner.get(&type_id).expect("Storage must have been initialized"),
            marker: PhantomData
        }
        
    }

    pub fn fetch_data<'a, T: FetchData<'a>>(&'a self, entity: E) -> <T as FetchData<'a>>::Item {
        T::fetch(entity, self)
    }
}

#[cfg(test)]
mod table_test {
    use crate::entity::{Entity, EntityManager};
    use crate::{create_entity, Component};
    use super::{Table, FetchData};

    create_entity! { TestId }

    #[derive(Default)]
    struct Context {
        manager: EntityManager<TestId>,
        data: Table<TestId>,
    }

    impl Context {
        fn insert(&mut self, component: impl Component) {
            let id = self.manager.create();
            component.register(id, &mut self.data);
        }
    }

    #[test]
    fn insert_get() {
        let mut cx = Context::default();
        for i in 0..10 {
            cx.insert((i.to_string(), i, i as f32));
        }
        let query_one = cx.data.query::<String>();
        query_one.into_iter().for_each(|q| println!("{q}"));

        // let query_two = cx.data.query::<(String, i32)>();
        // query_two.into_iter().for_each(|q| println!("{q:?}"));
    }

    #[test]
    fn fetch() {
        let mut cx = Context::default();
        for i in 0..10 {
            cx.insert((i.to_string(), i, i as f32));
        }
        let fetch_type = <(String, f32)>::fetch(TestId(1), &cx.data);
        let fetch_table = cx.data.fetch_data::<(String, f32)>(TestId(1));
        assert_eq!(fetch_type, fetch_table);
    }
}
