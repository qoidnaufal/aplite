use crate::data::component::{Component, ComponentTuple};
use crate::data::table::{ComponentTable, ComponentStorage};
use crate::buffer::TypeErasedBuffer;
use crate::sparse_set::indices::SparseSetIndex;

/// Query on many component type
pub struct Query<'a, Q: QueryData<'a>> {
    marker: std::marker::PhantomData<fn() -> &'a Q>,
}

struct QueryState {}

pub trait Queryable<'a> {
    type Item: Component;
    type Output: 'a;
}

impl<'a, T: Component> Queryable<'a> for &'a T {
    type Item = T;
    type Output = &'a T;
}

impl<'a, T: Component> Queryable<'a> for &'a mut T {
    type Item = T;
    type Output = &'a mut T;
}

pub trait QueryData<'a>: Sized {
    type Fetch: ComponentTuple;

    fn get_component_table() -> &'a ComponentTable;
}
