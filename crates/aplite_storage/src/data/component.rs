use std::any::TypeId;
use std::marker::PhantomData;
use std::slice::Iter;

use super::table::Table;
use super::array::Array;

use crate::entity::Entity;

/// Query on many component type
pub struct Query<'a, Qd: QueryData<'a>> {
    pub(crate) inner: Option<Qd::Iter>,
    pub(crate) marker: PhantomData<fn() -> &'a Qd>,
}

impl<'a, Qd: QueryData<'a>> Query<'a, Qd> {
    pub fn new<E: Entity + 'static>(source: &'a Table<E>) -> Self {
        Self {
            inner: Qd::query(source).ok(),
            marker: PhantomData,
        }
    }
}

/// Query (and iterate) one component type
pub struct QueryOne<'a, T> {
    pub(crate) inner: Option<Iter<'a, T>>,
}

impl<'a, T: 'static> QueryOne<'a, T> {
    pub(crate) fn new<E: Entity + 'static>(table: &'a Table<E>) -> Self {
        Self {
            inner: table.inner
                .get(&TypeId::of::<T>())
                .and_then(|any| any.downcast_ref::<Array<E, T>>())
                .map(|column| column.data.iter())
        }
    }
}

impl<'a, T: 'static> Iterator for QueryOne<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .as_mut()
            .and_then(|filter_map| filter_map.next())
    }
}

pub trait Component: Sized + 'static {
    fn register<E: Entity + 'static>(self, entity: &E, table: &mut Table<E>);
}

pub trait FetchData<'a> {
    type Item: 'a;

    fn fetch<E: Entity + 'static>(entity: &'a E, source: &'a Table<E>) -> Option<Self::Item>;
}

pub trait QueryData<'a> {
    type Iter;

    fn query<E: Entity + 'static>(source: &'a Table<E>) -> Result<Self::Iter, InvalidComponent>;
}

pub trait Remove {
    type Removed;

    fn remove<E: Entity + 'static>(entity: E, source: &mut Table<E>) -> Option<Self::Removed>;
}

macro_rules! component {
    ($($name:ident),*) => {
        impl<$($name: 'static),*> Component for ($($name,)*) {
            fn register<En: Entity + 'static>(self, entity: &En, table: &mut Table<En>) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                $(table.insert_one(entity, $name);)*
            }
        }

        impl<'a, $($name: 'static),*> FetchData<'a> for ($(&'a $name,)*) {
            type Item = ($(&'a $name,)*);

            fn fetch<En: Entity + 'static>(entity: &'a En, source: &'a Table<En>) -> Option<Self::Item> {
                Some(($(
                    source.inner
                        .get(&TypeId::of::<$name>())
                        .and_then(|any| any.downcast_ref::<Array<En, $name>>())
                        .and_then(|column| column.get(entity))?,
                )*))
            }
        }

        impl<$($name: 'static),*> Remove for ($($name,)*) {
            type Removed = ($($name,)*);

            fn remove<En: Entity + 'static>(entity: En, source: &mut Table<En>) -> Option<Self::Removed> {
                Some(($(
                    source.inner
                        .get_mut(&TypeId::of::<$name>())
                        .and_then(|any| any.downcast_mut::<Array<En, $name>>())
                        .and_then(|column| column.remove(entity))?,
                )*))
            }
        }
    };
}

macro_rules! query {
    ($($name:ident),*) => {
        impl<'a, $($name: 'static),*> QueryData<'a> for ($(&'a $name,)*) {
            type Iter = ($(Iter<'a, $name>,)*);

            fn query<En: Entity + 'static>(source: &'a Table<En>) -> Result<Self::Iter, InvalidComponent> {
                Ok(($(
                    source.inner
                        .get(&TypeId::of::<$name>())
                        .and_then(|any| any.downcast_ref::<Array<En, $name>>())
                        .map(|dense| dense.data.iter())
                        .ok_or(InvalidComponent(stringify!($name)))?,
                )*))
            }
        }

        impl<'a, $($name: 'static),*> Iterator for Query<'a, ($(&'a $name,)*)> {
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

// fn downcast<'a, T: 'static>(b: &'a Box<dyn Any>) -> Option<&'a T> {
//     b.downcast_ref::<T>()
// }
// type FnDownCast<'a, T> = fn(&'a Box<dyn Any>) -> Option<&'a T>;
// type FilteredQuery<'a, T> = FilterMap<Iter<'a, Box<dyn Any>>, FnDownCast<'a, T>>;
