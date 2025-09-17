use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::iter::FilterMap;

use super::table::Table;

use crate::entity::Entity;
use crate::data::dense_column::DenseColumnIter;

/// Query on many component type
pub struct Query<'a, E: Entity, Qd: QueryData<'a, E>> {
    pub(crate) inner: Option<Qd::Iter>,
    pub(crate) marker: PhantomData<fn() -> &'a Qd>,
}

impl<'a, E: Entity, Qd: QueryData<'a, E>> Query<'a, E, Qd> {
    pub fn new(source: &'a Table<E>) -> Self {
        Self {
            inner: Qd::query(source).ok(),
            marker: PhantomData,
        }
    }
}

/// Query (and iterate) one component type
pub struct QueryOne<'a, E: Entity, T> {
    pub(crate) inner: Option<FilteredQuery<'a, E, T>>,
}

fn downcast<'a, T: 'static>((_, b): (usize, &'a Box<dyn Any>)) -> Option<&'a T> {
    b.downcast_ref::<T>()
}

type FnDownCast<'a, T> = fn((usize, &'a Box<dyn Any>)) -> Option<&'a T>;
type FilteredQuery<'a, E, T> = FilterMap<DenseColumnIter<'a, E, Box<dyn Any>>, FnDownCast<'a, T>>;

impl<'a, E: Entity, T: 'static> QueryOne<'a, E, T> {
    pub(crate) fn new(table: &'a Table<E>) -> Self {
        Self {
            inner: table.inner
                .get(&TypeId::of::<T>())
                .map(|col| col.iter().filter_map(downcast as FnDownCast<'a, T>)),
        }
    }
}

impl<'a, E: Entity, T: 'static> Iterator for QueryOne<'a, E, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .as_mut()
            .and_then(|filter_map| filter_map.next())
    }
}

pub trait Component: Sized + 'static {
    fn register<E: Entity>(self, entity: &E, table: &mut Table<E>);
}

pub trait FetchData<'a> {
    type Item;

    fn fetch<E: Entity>(entity: &'a E, source: &'a Table<E>) -> Option<Self::Item>;
}

pub trait QueryData<'a, E: Entity> {
    type Iter;

    fn query(source: &'a Table<E>) -> Result<Self::Iter, InvalidComponent>;
}

macro_rules! component {
    ($($name:ident),*) => {
        impl<$($name: 'static),*> Component for ($($name,)*) {
            fn register<En: Entity>(self, entity: &En, table: &mut Table<En>) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                $(table.insert_one(entity, $name);)*
            }
        }

        impl<'a, $($name: 'static),*> FetchData<'a> for ($(&'a $name,)*) {
            type Item = ($(&'a $name,)*);

            fn fetch<En: Entity>(entity: &'a En, source: &'a Table<En>) -> Option<Self::Item> {
                Some(($(
                    source.inner
                        .get(&TypeId::of::<$name>())
                        .and_then(|row| {
                            row.get(entity)
                                .and_then(|any| any.downcast_ref::<$name>())
                        })?,
                )*))
            }
        }
    };
}

macro_rules! query {
    ($($name:ident),*) => {
        impl<'a, En: Entity, $($name: 'static),*> QueryData<'a, En> for ($(&'a $name,)*) {
            type Iter = ($(FilterMap<DenseColumnIter<'a, En, Box<dyn Any>>, FnDownCast<'a, $name>>,)*);

            fn query(source: &'a Table<En>) -> Result<Self::Iter, InvalidComponent> {
                Ok(($(
                    source.inner
                        .get(&TypeId::of::<$name>())
                        .map(|dense| {
                            dense.iter().filter_map(downcast as FnDownCast<'a, $name>)
                        })
                        .ok_or(InvalidComponent(stringify!($name)))?,
                )*))
            }
        }

        impl<'a, En: Entity, $($name: 'static),*> Iterator for Query<'a, En, ($(&'a $name,)*)> {
            type Item = ($(&'a $name,)*);

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
    U, V, W, X, Y, Z
);

impl_tuple_macro!(
    query,
    A, B, C, D, E,
    F, G, H, I, J,
    K, L, M, N, O,
    P, Q, R, S, T,
    U, V, W, X, Y, Z
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
