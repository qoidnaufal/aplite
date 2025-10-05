use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::marker::PhantomData;

use super::sparse_index::SparseIndices;
use super::component::{
    Component,
    IntoComponent,
    Query,
    QueryOne,
    QueryData,
    Queryable,
};

use crate::entity::Entity;

pub struct Table<E: Entity + 'static> {
    /// internally it's Vec<UnsafeCell<T>>,
    pub(crate) inner: HashMap<TypeId, Box<dyn Any>>,
    pub(crate) ptr: SparseIndices<E>,
    pub(crate) entities: Vec<E>,
    marker: PhantomData<E>,
}

impl<E: Entity + 'static> Default for Table<E> {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<E: Entity + 'static> Table<E> {
    /// Set the capacity for the contained entity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::default(),
            ptr: SparseIndices::default(),
            entities: Vec::with_capacity(capacity),
            marker: PhantomData,
        }
    }

    pub fn register_component(&mut self, entity: &E, component: impl IntoComponent) {
        let insert_index = self.entities.len();
        self.ptr.resize_if_needed(entity);
        self.ptr.set_index(entity.index(), insert_index);
        self.entities.push(*entity);
        component.into_component().register(entity, self);
    }

    pub fn remove<C: Component>(&mut self, entity: E) -> Option<C::Item> {
        C::remove(entity, self)
    }

    pub fn remove_into_component<IC: IntoComponent>(&mut self, entity: E) -> Option<<<IC as IntoComponent>::Item as Component>::Item> {
        IC::Item::remove(entity, self)
    }

    pub(crate) fn insert<T: 'static>(&mut self, entity: &E, value: T) {
        if let Some(vec) = self.inner
            .entry(TypeId::of::<T>())
            .or_insert(Box::new(Vec::<UnsafeCell<T>>::new()))
            .downcast_mut::<Vec<UnsafeCell<T>>>()
        {
            if let Some(index) = self.ptr.get_data_index(entity) {
                if let Some(data) = vec.get_mut(index) {
                    *data.get_mut() = value;
                    return;
                }
            }

            vec.push(UnsafeCell::new(value));
        }
    }

    pub fn update<T: 'static>(&mut self, entity: &E, value: T) {
        if let Some(any) = self.inner.get_mut(&TypeId::of::<T>()) {
            if let Some(vec) = any.downcast_mut::<Vec<UnsafeCell<T>>>() {
                if let Some(index) = self.ptr.get_data_index(entity) {
                    *vec[index].get_mut() = value;
                }
            }
        }
    }

    pub fn update_unsafe<T: 'static>(&self, entity: &E, value: T) {
        if let Some(any) = self.inner.get(&TypeId::of::<T>()) {
            if let Some(vec) = any.downcast_ref::<Vec<UnsafeCell<T>>>() {
                if let Some(index) = self.ptr.get_data_index(entity) {
                    unsafe {
                        *&mut *vec[index].get() = value;
                    }
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn contains(&self, entity: &E) -> bool {
        self.entities.contains(entity)
    }

    pub fn fetch_one<'a, Q: QueryData<'a>>(&'a self, entity: &E) -> Option<Q::Output> {
        self.inner
            .get(&TypeId::of::<Q::Item>())
            .and_then(|any| any.downcast_ref::<Vec<UnsafeCell<Q::Item>>>())
            .and_then(|arr| {
                self.ptr
                    .get_index(entity)
                    .map(|index| Q::get(&arr[index.index()]))
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

    pub fn clear(&mut self) {
        self.inner.clear();
        self.entities.clear();
        self.ptr.reset();
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
mod table_test {
    use crate::entity::{Entity, EntityManager};
    use crate::create_entity;
    use super::Table;
    use crate::IntoComponent;

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
            cx.table.register_component(&id, (i.to_string(), (i + 1) * -1, i as f32));
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
            cx.table.register_component(&id, (i.to_string(), (i + 1) * -1, i as f32));
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

    #[test]
    fn insert_tuple_component() {
        let mut cx = Context::default();
        for i in 0..10 {
            let id = cx.manager.create();
            cx.table.register_component(&id, (i.to_string(), (i, i + 1)));
        }

        let query = cx.table.query::<(&String, &(i32, i32))>();

        for (s, t) in query {
            assert!(std::any::type_name_of_val(s).contains("String"));
            assert_eq!(std::any::type_name_of_val(t), "(i32, i32)");
        }
    }

    #[derive(Default)]
    struct Dummy {
        name: String,
        age: u8,
        score: f32,
    }

    impl IntoComponent for Dummy {
        type Item = (u8, String, f32);
        fn into_component(self) -> Self::Item {
            let Self {
                name,
                age,
                score,
            } = self;
            (age, name, score)
        }
    }

    #[test]
    fn into_component() {
        let mut cx = Context::default();
        for i in 0..10 {
            let id = cx.manager.create();
            let dummy = Dummy {
                name: i.to_string(),
                age: i,
                score: i as _,
            };
            cx.table.register_component(&id, dummy);
        }
        let query = cx.table.query::<(&String, &f32, &u8)>();
        for (s, f, u) in query {
            assert!(std::any::type_name_of_val(s).contains("String"));
            assert!(std::any::type_name_of_val(f).contains("f32"));
            assert!(std::any::type_name_of_val(u).contains("u8"));
        }

        let removed = cx.table.remove_into_component::<Dummy>(TestId(6));
        assert!(removed.is_some_and(|(u, s, f)| {
            std::any::type_name_of_val(&s).contains("String") &&
            std::any::type_name_of_val(&f).contains("f32") &&
            std::any::type_name_of_val(&u).contains("u8")
        }));
    }
}
