use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;

use super::sparse_index::SparseIndices;
use super::component::{
    Component,
    IntoComponent,
    Query,
    QueryData,
};

use crate::entity::EntityId;

pub struct Table {
    /// internally it's `Vec<UnsafeCell<T>>`,
    pub(crate) inner: HashMap<TypeId, Box<dyn Any>>,
    pub(crate) ptr: SparseIndices,
    pub(crate) entities: Vec<EntityId>,
    pub(crate) components: Vec<TypeId>,
}

impl Default for Table {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl Table {
    /// Set the capacity for the contained entity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::default(),
            ptr: SparseIndices::default(),
            entities: Vec::with_capacity(capacity),
            components: Vec::new(),
        }
    }

    pub fn register_component(&mut self, id: &EntityId, component: impl IntoComponent) {
        let insert_index = self.entities.len();
        self.ptr.set_index(id, insert_index);
        self.entities.push(*id);
        component.into_component().register(id, self);
    }

    pub(crate) fn insert<T: 'static>(&mut self, id: &EntityId, value: T) {
        let type_id = TypeId::of::<T>();
        let vec = self.inner
            .entry(type_id)
            .or_insert(Box::new(Vec::<UnsafeCell<T>>::new()))
            .downcast_mut::<Vec<UnsafeCell<T>>>()
            .unwrap();

        if let Some(index) = self.ptr.get_data_index(id) {
            if let Some(data) = vec.get_mut(index) {
                *data.get_mut() = value;
                return;
            }
        }

        vec.push(UnsafeCell::new(value));
        self.components.push(type_id);
    }

    pub fn update<T: 'static>(&mut self, id: &EntityId, value: T) {
        if let Some(any) = self.inner.get_mut(&TypeId::of::<T>()) {
            if let Some(vec) = any.downcast_mut::<Vec<UnsafeCell<T>>>() {
                if let Some(index) = self.ptr.get_data_index(id) {
                    *vec[index].get_mut() = value;
                }
            }
        }
    }

    pub fn update_unsafe<T: 'static>(&self, id: &EntityId, value: T) {
        if let Some(any) = self.inner.get(&TypeId::of::<T>()) {
            if let Some(vec) = any.downcast_ref::<Vec<UnsafeCell<T>>>() {
                if let Some(index) = self.ptr.get_data_index(id) {
                    unsafe {
                        *&mut *vec[index].get() = value;
                    }
                }
            }
        }
    }

    pub fn remove<C: Component>(&mut self, id: EntityId) -> Option<C::Item> {
        C::remove(id, self)
    }

    pub fn remove_into_component<IC: IntoComponent>(&mut self, id: EntityId)
    -> Option<<<IC as IntoComponent>::Item as Component>::Item> {
        IC::Item::remove(id, self)
    }

    pub fn query<'a, Q: QueryData<'a>>(&'a self) -> Query<'a, Q> {
        Query::new(self)
    }

    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn contains(&self, id: &EntityId) -> bool {
        self.entities.contains(id)
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
    use crate::entity::{IdManager, EntityId};
    use super::Table;
    use crate::IntoComponent;

    #[derive(Default)]
    struct Context {
        manager: IdManager,
        table: Table,
    }

    #[test]
    fn query_one() {
        let mut cx = Context::default();
        for i in 0..10 {
            let id = cx.manager.create();
            cx.table.insert(&id, i.to_string());
        }
        let query_one = cx.table.query::<&String>();
        assert_eq!(query_one.iter().count(), 10);
    }

    #[test]
    fn query() {
        let mut cx = Context::default();
        for i in 0..10 {
            let id = cx.manager.create();
            cx.table.register_component(&id, (i.to_string(), (i + 1) * -1, i as f32));
        }

        let query = cx.table.query::<(&String, &mut f32, &i32)>();

        for (s, f, i) in query.iter() {
            *f = *i as f32;
            assert!(std::any::type_name_of_val(s).contains("String"));
            assert!(std::any::type_name_of_val(f).contains("f32"));
            assert!(std::any::type_name_of_val(i).contains("i32"));
        }

        let query_f32 = cx.table.query::<&f32>();

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

        let id = EntityId::new(3, 0);
        let items = cx.table.query::<(&i32, &mut String)>().get(&id);
        assert!(items.is_some());

        if let Some((i, s)) = items {
            *s = i.pow(2).to_string();
        }

        let fetch = cx.table.query::<&String>().get(&id);
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

    #[test]
    fn into_component() {
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
        let query = crate::Query::<'_, (&String, &f32, &u8)>::new(&cx.table);
        for (s, f, u) in &query {
            assert!(std::any::type_name_of_val(s).contains("String"));
            assert!(std::any::type_name_of_val(f).contains("f32"));
            assert!(std::any::type_name_of_val(u).contains("u8"));
        }
        let removed = cx.table.remove_into_component::<Dummy>(EntityId::new(6, 0));
        assert!(removed.is_some_and(|(u, s, f)| {
            std::any::type_name_of_val(&s).contains("String") &&
            std::any::type_name_of_val(&f).contains("f32") &&
            std::any::type_name_of_val(&u).contains("u8")
        }));
    }
}

// crate::create_entity! { TableId }

// pub struct DataStorage<E: Entity + 'static> {
//     tables: crate::IndexMap<TableId, Table<E>>,
// }
