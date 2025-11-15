use std::cell::UnsafeCell;

use crate::Component;
use crate::sparse_set::indices::SparseSetIndex;

/// Query on many component type
pub struct Query<'a, Q: QueryData<'a>> {
    pub(crate) ptr: &'a [SparseSetIndex],
    marker: std::marker::PhantomData<Q>,
}

pub struct QueryIter<'a, Q: QueryData<'a>> {
    marker: std::marker::PhantomData<fn() -> &'a Q>
}

pub trait Queryable<'a> {
    type Item: Component;
    type Output: 'a;

    /// Convert `UnsafeCell<T>` to `&T` or `&mut T`.
    fn convert(item: &UnsafeCell<Self::Item>) -> Self::Output;
}

impl<'a, T: Component> Queryable<'a> for &'a T {
    type Item = T;
    type Output = &'a T;

    fn convert(item: &UnsafeCell<Self::Item>) -> Self::Output {
        unsafe { &*item.get() }
    }
}

impl<'a, T: Component> Queryable<'a> for &'a mut T {
    type Item = T;
    type Output = &'a mut T;

    fn convert(item: &UnsafeCell<Self::Item>) -> Self::Output {
        unsafe { &mut *item.get() }
    }
}

pub trait QueryData<'a>: Sized {}

pub(crate) fn map_query<'a, Q: Queryable<'a>>(cell: &'a UnsafeCell<Q::Item>) -> Q::Output {
    Q::convert(cell)
}

pub(crate) type FnMapQuery<'a, Q> =
    fn(&'a UnsafeCell<<Q as Queryable<'a>>::Item>) -> <Q as Queryable<'a>>::Output;

macro_rules! query {
    ($($name:ident),*) => {
        impl<'a, $($name: Queryable<'a>),*> QueryData<'a> for ($($name,)*) {}
    };
}

macro_rules! query_one {
    ($name:ident) => {
        impl<'a, $name: Queryable<'a>> QueryData<'a> for $name {}
    };
}

use crate::impl_tuple_macro;

impl_tuple_macro!(
    query,
    A, B, C, D, E,
    F, G, H, I, J
    // K, L, M, N, O,
    // P, Q, R, S, T,
    // U, V, W, X, Y,
    // Z
);

query_one!(A);
