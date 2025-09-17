use std::collections::HashMap;
use std::any::{Any, TypeId};

use super::dense_column::DenseColumn;
use super::component::{Component, Query, QueryOne, QueryData, FetchData};

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

    pub fn insert<C: Component>(&mut self, entity: &E, component: C) {
        component.register(entity, self);
    }

    pub fn insert_one<T: Any + 'static>(&mut self, entity: &E, value: T) {
        self.inner
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(entity, Box::new(value));
    }

    pub fn get<T: 'static>(&self, entity: &E) -> Option<&T> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|dense| {
                dense.get(entity)
                    .and_then(|any| any.downcast_ref())
            })
    }

    pub fn get_mut<T: 'static>(&mut self, entity: &E) -> Option<&mut T> {
        self.inner
            .get_mut(&TypeId::of::<T>())
            .and_then(|dense| {
                dense.get_mut(entity)
                    .and_then(|any| any.downcast_mut())
            })
    }

    pub fn fetch<'a, Fd: FetchData<'a>>(&'a self, entity: &'a E) -> Option<<Fd as FetchData<'a>>::Item> {
        Fd::fetch(entity, self)
    }

    pub fn query_one<T: 'static>(&self) -> QueryOne<'_, E, T> {
        QueryOne::new(self)
    }

    pub fn query<'a, Qd: QueryData<'a, E>>(&'a self) -> Query<'a, E, Qd> {
        Query::new(self)
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
            component.register(&id, &mut self.data);
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

        let query = cx.data.query::<(&String, &f32, &i32)>();

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

        let fetch_one = <(&i32,)>::fetch(&TestId(3), &cx.data);
        assert!(fetch_one.is_some());

        let fetch_many_from_type = <(&String, &f32)>::fetch(&TestId(1), &cx.data);
        let fetch_many_from_table = cx.data.fetch::<(&String, &f32)>(&TestId(1));
        assert_eq!(fetch_many_from_type, fetch_many_from_table);
    }
}
