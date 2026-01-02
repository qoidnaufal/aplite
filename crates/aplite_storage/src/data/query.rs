use std::ptr::NonNull;
use aplite_bitset::Bitset;

use crate::data::component::Component;
use crate::data::component_storage::{ComponentStorage, MarkedBuffer};
use crate::entity::EntityId;

/*
#########################################################
#
# Query
#
#########################################################
*/

pub struct Query<'a, Q: QueryData> {
    source: &'a ComponentStorage,
    matched_archetype_ids: Bitset,
    marker: std::marker::PhantomData<Q>,
}

impl<'a, Q> Query<'a, Q>
where
    Q: QueryData,
{
    pub fn new(source: &'a ComponentStorage) -> Self {
        let matched_component_ids = Q::matched_component_ids(source).unwrap_or_default();

        Self {
            source,
            matched_archetype_ids: source.get_archetype_ids(matched_component_ids),
            marker: std::marker::PhantomData,
        }
    }

    pub fn update_query_state(&mut self) {
        let matched_component_ids = Q::matched_component_ids(self.source).unwrap_or_default();
        self.matched_archetype_ids = self.source.get_archetype_ids(matched_component_ids);
    }

    pub fn entities(&self) -> Box<[&EntityId]> {
        self.matched_archetype_ids
            .iter()
            .flat_map(|id| &self.source.archetype_tables[(1 << id) as usize].entities)
            .collect()
    }

    pub fn buffers(&'a self) -> <Q as QueryData>::Buffer<'a> {
        Q::get_buffer(self.source, self.matched_archetype_ids)
    }

    pub fn iter(&'a self) -> QueryIter<'a, Q> {
        QueryIter {
            current: None,
            buffer: self.buffers(),
            buffer_counter: 0,
            counter: 0,
            len: 0,
            marker: std::marker::PhantomData,
        }
    }
}

/*
#########################################################
#
# QueryData
#
#########################################################
*/

pub trait QueryData {
    type Item: 'static;
    type Fetch<'a>;
    type State;
    type Buffer<'a>;

    fn get<'a>(input: Self::State) -> Self::Fetch<'a>;

    fn matched_component_ids(source: &ComponentStorage) -> Option<Bitset>;

    fn get_buffer<'a>(source: &'a ComponentStorage, table_ids: Bitset) -> Self::Buffer<'a>;
}

impl<'a, T: Component + 'static> QueryData for &'a T {
    type Item = T;
    type Fetch<'b> = &'a T;
    type State = NonNull<Self::Item>;
    type Buffer<'b> = Box<[MarkedBuffer<'b, Self::Fetch<'b>>]>;

    fn get<'b>(input: Self::State) -> Self::Fetch<'b> {
        unsafe {
            &*input.as_ptr()
        }
    }

    fn matched_component_ids(source: &ComponentStorage) -> Option<Bitset> {
        source.get_component_id::<Self::Item>()
            .map(|id| Bitset::new(1 << id.0))
    }

    fn get_buffer<'b>(source: &'b ComponentStorage, table_ids: Bitset) -> Self::Buffer<'b> {
        source.get_marked_buffers(table_ids)
    }
}

impl<'a, T: Component + 'static> QueryData for &'a mut T {
    type Item = T;
    type Fetch<'b> = &'a mut T;
    type State = NonNull<Self::Item>;
    type Buffer<'b> = Box<[MarkedBuffer<'b, Self::Fetch<'b>>]>;

    fn get<'b>(input: Self::State) -> Self::Fetch<'b> {
        unsafe {
            &mut *input.as_ptr()
        }
    }

    fn matched_component_ids(source: &ComponentStorage) -> Option<Bitset> {
        source.get_component_id::<Self::Item>()
            .map(|id| Bitset::new(1 << id.0))
    }

    fn get_buffer<'b>(source: &'b ComponentStorage, table_ids: Bitset) -> Self::Buffer<'b> {
        source.get_marked_buffers(table_ids)
    }
}

/*
#########################################################
#
# QueryIter
#
#########################################################
*/

pub struct QueryIter<'a, Q: QueryData> {
    pub(crate) current: Option<Q::State>,
    pub(crate) buffer: Q::Buffer<'a>,
    pub(crate) buffer_counter: usize,
    pub(crate) counter: usize,
    pub(crate) len: usize,
    pub(crate) marker: std::marker::PhantomData<Q::Fetch<'a>>,
}

impl<'a, T: Component + 'static> Iterator for QueryIter<'a, &'a T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(raw) = self.current {
            if self.counter < self.len {
                let next = unsafe { raw.add(self.counter) };
                self.counter += 1;
                return Some(<&'a T as QueryData>::get(next));
            }
        }

        let buffer = self.buffer.get(self.buffer_counter)?;
        self.current = Some(buffer.start);
        self.buffer_counter += 1;
        self.counter = 0;
        self.len = buffer.len;
        self.next()
    }
}

impl<'a, T: Component + 'static> Iterator for QueryIter<'a, &'a mut T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(raw) = self.current {
            if self.counter < self.len {
                let next = unsafe { raw.add(self.counter) };
                self.counter += 1;
                return Some(<&'a mut T as QueryData>::get(next));
            }
        }

        let buffer = self.buffer.get(self.buffer_counter)?;
        self.current = Some(buffer.start);
        self.buffer_counter += 1;
        self.counter = 0;
        self.len = buffer.len;
        self.next()
    }
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
