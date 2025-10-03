use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

use super::array::Array;
use super::component::{
    Component,
    InvalidComponent,
    Query,
    QueryOne,
    QueryData,
    Queryable,
};

use crate::entity::Entity;

pub struct Table<E: Entity + 'static> {
    pub(crate) inner: HashMap<TypeId, Box<dyn Any>>,
    marker: PhantomData<E>,
}

impl<E: Entity + 'static> Default for Table<E> {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<E: Entity + 'static> Table<E> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
            marker: PhantomData,
        }
    }

    pub fn register<C: Component>(&mut self, entity: &E, component: C) {
        component.register(entity, self);
    }

    pub fn remove<C: Component>(&mut self, entity: E) -> Result<C::Item, InvalidComponent> {
        C::remove(entity, self)
    }

    pub fn insert<T: 'static>(&mut self, entity: &E, value: T) {
        if let Some(array) = self.inner
            .entry(TypeId::of::<T>())
            .or_insert(Box::new(Array::<E, T>::default()))
            .downcast_mut::<Array<E, T>>()
        {
            array.insert(entity, value);
        }
    }

    pub fn fetch_one<'a, Q: QueryData<'a>>(&'a self, entity: &E) -> Option<Q::Output> {
        self.inner
            .get(&TypeId::of::<Q::Item>())
            .and_then(|any| any.downcast_ref::<Array<E, Q::Item>>())
            .and_then(|arr| {
                arr.get_raw(entity)
                    .map(|cell| Q::get(cell))
            })
    }

    pub fn fetch<'a, Q: Queryable<'a>>(&'a self, entity: &E) -> Option<Q::Fetch> {
        Q::fetch(entity, self)
    }

    pub fn query_one<'a, Q: QueryData<'a>>(&'a self) -> QueryOne<'a, Q> {
        QueryOne::new(self)
    }

    pub fn query<'a, Q: Queryable<'a>>(&'a self) -> Query<'a, Q> {
        Query::new(self)
    }
}

#[cfg(test)]
mod table_test {
    use crate::entity::{Entity, EntityManager};
    use crate::create_entity;
    use super::Table;

    create_entity! { TestId }

    #[derive(Default)]
    struct Context {
        manager: EntityManager<TestId>,
        table: Table<TestId>,
    }

    #[test]
    fn query_one() {
        let mut cx = Context::default();
        for i in 0..10 {
            let id = cx.manager.create();
            cx.table.insert(&id, i.to_string());
        }
        let query_one = cx.table.query_one::<&String>();
        assert_eq!(query_one.count(), 10);
    }

    #[test]
    fn query() {
        let mut cx = Context::default();
        for i in 0..10 {
            let id = cx.manager.create();
            cx.table.register(&id, (i.to_string(), (i + 1) * -1, i as f32));
        }

        let query = cx.table.query::<(&String, &mut f32, &i32)>();

        for (s, f, i) in query {
            *f = *i as f32;
            assert!(std::any::type_name_of_val(s).contains("String"));
            assert!(std::any::type_name_of_val(f).contains("f32"));
            assert!(std::any::type_name_of_val(i).contains("i32"));
        }

        let query_f32 = cx.table.query_one::<&f32>();

        for f in query_f32 {
            assert!(f.is_sign_negative())
        }
    }

    #[test]
    fn fetch_component() {
        let mut cx = Context::default();
        for i in 0..10 {
            let id = cx.manager.create();
            cx.table.register(&id, (i.to_string(), (i + 1) * -1, i as f32));
        }

        let entity = TestId(3);
        let items = cx.table.fetch::<(&i32, &mut String)>(&entity);
        assert!(items.is_some());

        if let Some((i, s)) = items {
            *s = i.pow(2).to_string();
        }

        let fetch = cx.table.fetch_one::<&String>(&entity);
        assert!(fetch.is_some_and(|n| !n.starts_with('-')));
    }
}
