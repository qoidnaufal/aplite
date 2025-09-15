use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

use super::dense_column::DenseColumn;
use super::query::{Query, QueryOne, QueryData, FetchData};

use crate::entity::Entity;

pub struct Table<E: Entity> {
    pub(crate) inner: HashMap<TypeId, DenseColumn<E, Box<dyn Any>>>,
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

    pub fn fetch<'a, T: FetchData<'a, E>>(&'a self, entity: E) -> Option<<T as FetchData<'a, E>>::Item> {
        T::fetch(entity, self)
    }

    pub fn query_one<T: 'static>(&self) -> QueryOne<'_, E, T> {
        QueryOne::new(self)
    }

    pub fn query<'a, Qd: QueryData<'a>>(&'a self) -> Query<'a, Qd> {
        Query {
            inner: Qd::query(self),
            marker: PhantomData,
        }
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
    fn query_one() {
        let mut cx = Context::default();
        for i in 0..10 {
            cx.insert((i.to_string(),));
        }
        let query_one = cx.data.query_one::<String>();
        assert_eq!(query_one.count(), 10);
    }

    #[test]
    fn query() {
        let mut cx = Context::default();
        for i in 0..10 {
            cx.insert((i.to_string(), i, i as f32));
        }

        let query = cx.data.query::<(String, f32, i32)>();

        for (s, f, i) in query {
            assert!(std::any::type_name_of_val(s).contains("String"));
            assert!(std::any::type_name_of_val(f).contains("f32"));
            assert!(std::any::type_name_of_val(i).contains("i32"));
        }
    }

    #[test]
    fn fetch() {
        let mut cx = Context::default();
        for i in 0..10 {
            cx.insert((i.to_string(), i, i as f32));
        }

        let fetch_one = <(i32,)>::fetch(TestId(3), &cx.data);
        assert!(fetch_one.is_some());

        let fetch_many_from_type = <(String, f32)>::fetch(TestId(1), &cx.data);
        let fetch_many_from_table = cx.data.fetch::<(String, f32)>(TestId(1));
        assert_eq!(fetch_many_from_type, fetch_many_from_table);
    }
}
