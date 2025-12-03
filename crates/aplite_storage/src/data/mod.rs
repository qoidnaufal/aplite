use crate::entity::Entity;

pub(crate) mod component;
pub(crate) mod query;
pub(crate) mod table;

use component::{Component, ComponentBundle};
use query::{Queryable, QueryData};
use table::ComponentTable;

macro_rules! component_bundle {
    ($($name:ident),*) => {
        impl<$($name: Component),*> ComponentBundle for ($($name,)*) {
            type Item = ($($name,)*);

            fn insert_bundle(self, id: Entity, table: &mut ComponentTable) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                ($(table.insert(id, $name),)*);
            }
        }
    };
}

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
    component_bundle,
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

query_one!(A);
