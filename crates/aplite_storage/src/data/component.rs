use std::any::{TypeId, type_name};
use std::marker::PhantomData;
use std::slice::Iter;
use std::iter::Map;
use std::cell::UnsafeCell;

use super::table::Table;
use super::array::Array;

use crate::entity::Entity;

/// Query on many component type
pub struct Query<'a, Q: Queryable<'a>> {
    pub(crate) inner: Option<Q::Iter>,
    pub(crate) marker: PhantomData<fn() -> &'a Q>,
}

impl<'a, Q: Queryable<'a>> Query<'a, Q> {
    pub fn new<E: Entity + 'static>(source: &'a Table<E>) -> Self {
        Self {
            inner: Q::query(source),
            marker: PhantomData,
        }
    }
}

/// Query (and iterate) one component type
pub struct QueryOne<'a, Q: QueryData<'a>> {
    pub(crate) inner: Option<Map<Iter<'a, UnsafeCell<Q::Fetch>>, FnMapQuery<'a, Q>>>,
}

impl<'a, Q: QueryData<'a>> QueryOne<'a, Q> {
    pub(crate) fn new<E: Entity + 'static>(table: &'a Table<E>) -> Self {
        Self {
            inner: table.inner.get(&TypeId::of::<Q::Fetch>())
                .and_then(|any| any.downcast_ref::<Array<E, Q::Fetch>>())
                .map(|arr| arr.query_one::<Q>())
        }
    }
}

impl<'a, Q: QueryData<'a>> Iterator for QueryOne<'a, Q> {
    type Item = Q::Output;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .as_mut()
            .and_then(|i| i.next())
    }
}

pub trait Component: Sized + 'static {
    type Item;

    /// Register value(s) to the table
    fn register<E: Entity + 'static>(self, entity: &E, table: &mut Table<E>);

    /// Use this function carefully, or else this will mess up your data
    fn remove<E: Entity + 'static>(entity: E, source: &mut Table<E>) -> Result<Self::Item, InvalidComponent>;
}

pub trait QueryData<'a> {
    type Fetch: 'static;
    type Output: 'a;

    /// Convert `UnsafeCell<T>` to `&T` or `&mut T`.
    fn get(fetch: &UnsafeCell<Self::Fetch>) -> Self::Output;
}

impl<'a, T: 'static> QueryData<'a> for &'a T {
    type Fetch = T;
    type Output = &'a T;

    fn get(item: &UnsafeCell<Self::Fetch>) -> Self::Output {
        unsafe { &*item.get() }
    }
}

impl<'a, T: 'static> QueryData<'a> for &'a mut T {
    type Fetch = T;
    type Output = &'a mut T;

    fn get(item: &UnsafeCell<Self::Fetch>) -> Self::Output {
        unsafe { &mut *item.get() }
    }
}

pub trait Queryable<'a> {
    type Iter;

    fn query<E: Entity + 'static>(source: &'a Table<E>) -> Option<Self::Iter>;
}

macro_rules! component {
    ($($name:ident),*) => {
        impl<$($name: 'static),*> Component for ($($name,)*) {
            type Item = ($($name,)*);

            fn register<En: Entity + 'static>(self, entity: &En, table: &mut Table<En>) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                $(table.insert(entity, $name);)*
            }

            fn remove<En: Entity + 'static>(entity: En, source: &mut Table<En>) -> Result<Self::Item, InvalidComponent> {
                Ok(($(
                    source.inner
                        .get_mut(&TypeId::of::<$name>())
                        .and_then(|any| any.downcast_mut::<Array<En, $name>>())
                        .and_then(|array| array.remove(entity))
                        .ok_or(InvalidComponent(type_name::<$name>()))?,
                )*))
            }
        }
    };
}

pub(crate) fn map_query<'a, Qd: QueryData<'a>>(cell: &'a UnsafeCell<Qd::Fetch>) -> Qd::Output {
    Qd::get(cell)
}

pub(crate) type FnMapQuery<'a, Qd> =
    fn(&'a UnsafeCell<<Qd as QueryData<'a>>::Fetch>) -> <Qd as QueryData<'a>>::Output;

macro_rules! query {
    ($($name:ident),*) => {
        impl<'a, $($name: QueryData<'a>),*> Queryable<'a> for ($($name,)*) {
            type Iter = ($(Map<Iter<'a, UnsafeCell<$name::Fetch>>, FnMapQuery<'a, $name>>,)*);

            fn query<En: Entity + 'static>(source: &'a Table<En>) -> Option<Self::Iter> {
                Some(($(
                    source.inner
                        .get(&TypeId::of::<$name::Fetch>())
                        .and_then(|any| any.downcast_ref::<Array<En, $name::Fetch>>())
                        .map(|array| {
                            array.data
                                .iter()
                                .map(map_query::<'a, $name> as FnMapQuery<'a, $name>)
                        })?,
                )*))
            }
        }

        impl<'a, $($name: QueryData<'a>),*> Iterator for Query<'a, ($($name,)*)> {
            type Item = ($($name::Output,)*);

            fn next(&mut self) -> Option<Self::Item> {
                #[allow(non_snake_case)]
                let Some(($($name,)*)) = self.inner.as_mut() else { return None };
                Some(($($name.next()?,)*))
            }
        }
    };
}

macro_rules! impl_tuple_macro {
    ($macro:ident, $next:tt) => {
        $macro!{$next}
    };
    ($macro:ident, $next:tt, $($rest:tt),*) => {
        $macro!{$next, $($rest),*}
        impl_tuple_macro!{$macro, $($rest),*}
    };
}

impl_tuple_macro!(
    component,
    A, B, C, D, E,
    F, G, H, I, J,
    K, L, M, N, O,
    P, Q, R, S, T,
    U, V, W, X, Y,
    Z
);

impl_tuple_macro!(
    query,
    A, B, C, D, E,
    F, G, H, I, J,
    K, L, M, N, O,
    P, Q, R, S, T,
    U, V, W, X, Y,
    Z
);

#[derive(Debug)]
pub struct InvalidComponent(&'static str);

impl std::error::Error for InvalidComponent {}

impl std::fmt::Display for InvalidComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid component {}", self.0)
    }
}

impl PartialEq for InvalidComponent {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for InvalidComponent {}
