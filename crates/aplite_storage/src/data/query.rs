use crate::data::component::{Component, ComponentId};
use crate::data::table::{ComponentStorage, TableId};
use crate::data::bitset::Bitset;
use crate::entity::EntityId;

pub struct QueryIter<'a, Q: QueryData<'a>> {
    pub(crate) current: Option<Q::RawBuffer>,
    pub(crate) buffer: Q::Buffer,
    pub(crate) buffer_counter: usize,
    pub(crate) counter: usize,
    pub(crate) len: usize,
}

pub struct Query<'a, Q: QueryData<'a>> {
    source: &'a ComponentStorage,
    bitset: Bitset,
    table_ids: Box<[&'a TableId]>,
    marker: std::marker::PhantomData<Q>,
}

impl<'a, Q> Query<'a, Q>
where
    Q: QueryData<'a>,
{
    pub fn new(source: &'a ComponentStorage) -> Self {
        let component_ids = Q::component_ids(source)
            .map(|vec| vec.into_boxed_slice())
            .unwrap_or_default();

        let mut bitset = Bitset::new();

        component_ids.iter().for_each(|id| bitset.update(id.0));

        let table_ids = source.get_table_ids(bitset);

        Self {
            source,
            table_ids,
            bitset,
            marker: std::marker::PhantomData,
        }
    }

    pub fn entities(&self) -> Box<[&EntityId]> {
        self.table_ids.iter()
            .flat_map(|id| &self.source.tables[id.0].entities)
            .collect()
    }

    pub fn buffers(&'a self) -> Q::Buffer {
        Q::get_buffer(self.source, self.table_ids.as_ref())
    }

    pub fn iter(&'a self) -> QueryIter<'a, Q> {
        QueryIter {
            current: None,
            buffer: Q::get_buffer(self.source, self.table_ids.as_ref()),
            buffer_counter: 0,
            counter: 0,
            len: 0,
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

pub trait Queryable<'a> {
    type Item: Component + 'static;

    fn convert(input: *mut Self::Item) -> Self;
}

impl<'a, T: Component + 'static> Queryable<'a> for &'a T {
    type Item = T;

    fn convert(input: *mut Self::Item) -> Self {
        unsafe {
            &*input
        }
    }
}

impl<'a, T: Component + 'static> Queryable<'a> for &'a mut T {
    type Item = T;

    fn convert(input: *mut Self::Item) -> Self {
        unsafe {
            &mut *input
        }
    }
}

pub trait QueryData<'a> {
    type Items: crate::ComponentTuple;
    type RawBuffer;
    type Buffer;

    fn type_ids() -> Vec<std::any::TypeId>;

    fn component_ids(source: &ComponentStorage) -> Option<Vec<ComponentId>>;

    fn bitset(source: &ComponentStorage) -> Option<Bitset>;

    fn get_buffer(source: &'a ComponentStorage, table_ids: &[&'a TableId]) -> Self::Buffer;
}

/*
#########################################################
#
# Test
#
#########################################################
*/

#[cfg(test)]
mod query_test {
    use super::*;
    use crate::make_component;
    use crate::entity::EntityId;

    make_component!(struct Name(String));
    make_component!(struct Age(u32));
    make_component!(struct Salary(u32));
    make_component!(struct Cars(u32));

    #[test]
    fn mutable_query() {
        let mut storage = ComponentStorage::new();
        for i in 0..10 {
            storage.insert_component(EntityId::new(i), (Age(i), Salary(i)));
        }

        for (salary, age) in storage.query::<(&mut Salary, &Age)>().iter() {
            salary.0 = age.0 + 10
        }

        let query = storage.query::<(&Salary, &Age)>();
        assert!(query.iter().all(|(salary, age)| salary.0 - age.0 == 10));
    }

    #[test]
    fn different_archetypes() {
        let mut storage = ComponentStorage::new();

        for i in 0..10 {
            storage.insert_component(EntityId::new(i), (Age(i), Salary(i)));
        }

        for i in 10..20 {
            storage.insert_component(EntityId::new(i), (Age(i), Cars(i)));
        }

        let query1 = storage.query::<&Age>();
        let query2 = storage.query::<(&Age, &Cars)>();
        let query3 = storage.query::<&Cars>();

        // query1.iter().for_each(|age| println!("{age}"));
        // query2.iter().for_each(|(age, car)| println!("({age}, {car})"));
        // query3.iter().for_each(|car| println!("{car}"));

        assert!(query1.iter().count() > query2.iter().count());
        assert!(query1.iter().count() > query3.iter().count());
        assert_eq!(query2.iter().count(), query3.iter().count());
    }

    #[test]
    fn buffer_iter() {
        let mut storage = ComponentStorage::new();

        for i in 0..10 {
            storage.insert_component(
                EntityId::new(i),
                (Age(i), Name(i.to_string()), Salary(i), Cars(i))
            );
        }

        let query = storage.query::<(&Name, &Salary)>();
        let (name_buffers, salary_buffers) = query.buffers();

        assert_eq!(name_buffers.len(), salary_buffers.len());

        let mut counter = 0;

        for buffer in salary_buffers.iter() {
            for _ in buffer.iter() {
                counter += 1;
            }
        }

        assert_eq!(counter, 10);
    }
}
