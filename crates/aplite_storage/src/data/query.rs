use crate::data::component::{Component, ComponentBitset, ComponentId};
use crate::data::table::ComponentStorage;
use crate::entity::Entity;
use crate::sparse_set::SparsetKey;

pub struct Query<'a, Q> {
    source: &'a ComponentStorage,
    component_ids: Vec<ComponentId>,
    marker: std::marker::PhantomData<Q>,
}

pub struct QueryIter<'a, Q: QueryData<'a>> {
    pub(crate) buffer: Option<Q::Buffer>,
    pub(crate) entities: Option<&'a [Entity]>,
    pub(crate) counter: usize,
}

impl<'a, Q> Query<'a, Q>
where
    Q: QueryData<'a>,
{
    pub fn new(source: &'a ComponentStorage) -> Self {
        Self {
            source,
            component_ids: Q::component_ids(source).unwrap_or_default(),
            marker: std::marker::PhantomData,
        }
    }

    pub fn entities(&self) -> Option<&[Entity]> {
        self.component_ids.iter()
            .map(|component_id| {
                self.source
                    .indexer[component_id.index()]
                    .entities
                    .as_slice()
            })
            .min_by_key(|entities| entities.len())
    }

    pub fn buffers(&self) -> Option<Q::Buffer> {
        Q::get_buffer(self.source)
    }

    pub fn iter(&'a self) -> QueryIter<'a, Q> {
        QueryIter {
            buffer: Q::get_buffer(self.source),
            entities: self.entities(),
            counter: 0,
        }
    }
}

/*
#########################################################
#
# Traits
#
#########################################################
*/

pub trait Queryable {
    type Item: Component;

    fn convert(input: *mut Self::Item) -> Self;
}

impl<'a, T: Component> Queryable for &'a T {
    type Item = T;

    fn convert(input: *mut Self::Item) -> Self {
        unsafe {
            &*input
        }
    }
}

impl<'a, T: Component> Queryable for &'a mut T {
    type Item = T;

    fn convert(input: *mut Self::Item) -> Self {
        unsafe {
            &mut *input
        }
    }
}

pub trait QueryData<'a> {
    type Items;
    type Buffer;

    fn type_ids() -> Vec<std::any::TypeId>;

    fn component_ids(source: &ComponentStorage) -> Option<Vec<ComponentId>>;

    fn bitset(source: &ComponentStorage) -> Option<ComponentBitset>;

    fn get_buffer(source: &'a ComponentStorage) -> Option<Self::Buffer>;
}

#[cfg(test)]
mod query_test {
    use super::*;
    use crate::entity::Entity;
    use crate::make_component;

    make_component!(struct Age(u8));
    make_component!(struct Name(String));
    make_component!(struct Salary(usize));
    make_component!(struct Cars(usize));

    #[test]
    fn direct_query() {
        let mut storage = ComponentStorage::new();
        for i in 0..10 {
            storage.insert_component_tuple(
                Entity::with_id_version(i, 0),
                (Age(i as _), Name(i.to_string()), Salary(i as _), Cars(i as _))
            );
        }

        for salary in storage.query::<&mut Salary>().iter() {
            salary.0 += 10
        }

        let query = storage.query::<(&Salary, &Cars)>();
        assert!(query.iter().all(|(salary, cars)| salary.0 - cars.0 == 10));
    }

    #[test]
    fn buffer_iter() {
        let mut storage = ComponentStorage::new();
        for i in 0..10 {
            storage.insert_component_tuple(
                Entity::with_id_version(i, 0),
                (Age(i as _), Name(i.to_string()), Salary(i as _), Cars(i as _))
            );
        }

        let query = storage.query::<&mut Salary>();
        let buffer = query.buffers().unwrap();
        for salary in buffer.iter() {
            salary.0 += 10
        }

        let query = storage.query::<(&Salary, &Cars)>();
        assert!(query.iter().all(|(salary, cars)| salary.0 - cars.0 == 10));
    }
}
