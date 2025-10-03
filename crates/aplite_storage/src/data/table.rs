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

    pub fn insert<T: 'static>(&mut self, entity: &E, value: T) {
        if let Some(array) = self.inner
            .entry(TypeId::of::<T>())
            .or_insert(Box::new(Array::<E, T>::default()))
            .downcast_mut::<Array<E, T>>()
        {
            array.insert(entity, value);
        }
    }

    pub fn get<T: 'static>(&self, entity: &E) -> Option<&T> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|any| any.downcast_ref::<Array<E, T>>())
            .and_then(|dense| dense.get(entity))
    }

    pub fn get_mut<T: 'static>(&mut self, entity: &E) -> Option<&mut T> {
        self.inner
            .get_mut(&TypeId::of::<T>())
            .and_then(|any| any.downcast_mut::<Array<E, T>>())
            .and_then(|dense| dense.get_mut(entity))
    }

    pub fn query_one<'a, Q: QueryData<'a>>(&'a self) -> QueryOne<'a, Q> {
        QueryOne::new(self)
    }

    pub fn query<'a, Q: Queryable<'a>>(&'a self) -> Query<'a, Q> {
        Query::new(self)
    }

    pub fn register<C: Component>(&mut self, entity: &E, component: C) {
        component.register(entity, self);
    }

    pub fn remove<C: Component>(&mut self, entity: E) -> Result<C::Item, InvalidComponent> {
        C::remove(entity, self)
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

    // #[test]
    // fn removal() {
    //     let mut cx = Context::default();
    //     for i in 0..10 {
    //         cx.insert((i.to_string(), i, i as f32));
    //     }

    //     let removed = cx.table.remove::<(String, f32, i32)>(TestId(5));
    //     assert!(removed.is_some())
    // }
}
