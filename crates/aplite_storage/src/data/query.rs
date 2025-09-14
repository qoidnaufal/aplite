use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::iter::FilterMap;

use super::dense_row::DenseRow;
use super::table::Table;

use crate::entity::Entity;
use crate::iterator::DataStoreIter;

pub struct Query<'a, E: Entity, T> {
    pub(crate) inner: &'a DenseRow<E, Box<dyn Any>>,
    pub(crate) marker: PhantomData<T>,
}

impl<'a, E: Entity, T: 'static> Query<'a, E, T> {
    pub fn new(table: &'a Table<E>) -> Self {
        Self {
            inner: table.inner.get(&TypeId::of::<T>())
                .expect("Type needs to be valid, and has been registered"),
            marker: PhantomData,
        }
    }
}

fn downcast<'a, T: 'static>((_, b): (&usize, &'a Box<dyn Any>)) -> Option<&'a T> {
    b.downcast_ref()
}

type FnDownCast<'a, T> = fn((&'a usize, &'a Box<dyn Any>)) -> Option<&'a T>;

impl<'a, E: Entity, T: 'static> IntoIterator for Query<'a, E, T> {
    type Item = &'a T;
    type IntoIter = FilterMap<DataStoreIter<'a, Box<dyn Any>>, FnDownCast<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner
            .iter()
            .filter_map(downcast as FnDownCast<'a, T>)
    }
}

pub trait Component: Sized + 'static {
    fn register<E: Entity>(self, entity: E, table: &mut Table<E>);
}

pub trait FetchData<'a> {
    type Item;

    fn fetch<E: Entity>(entity: E, source: &'a Table<E>) -> Self::Item;
}

pub trait QueryData<'a> {
    type Item: 'a;

    fn query<E: Entity>(source: &'a Table<E>) -> Query<'a, E, Self::Item>;
}

macro_rules! component {
    ($($name:ident),*) => {
        impl<$($name: 'static),*> Component for ($($name,)*) {
            fn register<En: Entity>(self, entity: En, table: &mut Table<En>) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                $(table.insert(entity, $name);)*
            }
        }

        impl<'a, $($name: 'static),*> FetchData<'a> for ($($name,)*) {
            type Item = ($(Option<&'a $name>,)*);

            fn fetch<En: Entity>(entity: En, source: &'a Table<En>) -> Self::Item {
                ($(
                    source.inner
                        .get(&std::any::TypeId::of::<$name>())
                        .and_then(|row| {
                            row.get(entity).and_then(|any| any.downcast_ref::<$name>())
                        }),
                )*)
            }
        }

        impl<'a, $($name: 'static),*> QueryData<'a> for ($($name,)*) {
            type Item = ($(&'a $name,)*);

            fn query<En: Entity>(_source: &'a Table<En>) -> Query<'a, En, Self::Item> {
                todo!()
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
