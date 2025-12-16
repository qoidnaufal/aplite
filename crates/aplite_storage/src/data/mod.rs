use crate::entity::Entity;

pub(crate) mod component;
pub(crate) mod query;
pub(crate) mod table;

use component::{Component, ComponentBundle, ComponentBitset};
use query::{Queryable, QueryData};
use table::ComponentStorage;

macro_rules! component_bundle {
    ($($name:ident),*) => {
        impl<$($name: Component),*> ComponentBundle for ($($name,)*) {
            type Item = ($($name,)*);

            fn bitset(storage: &ComponentStorage) -> Option<ComponentBitset> {
                let mut bitset = ComponentBitset::new();
                ($(bitset.update(storage.get_component_id::<$name>()?),)*);
                Some(bitset)
            }

            fn insert_bundle(self, id: Entity, storage: &mut ComponentStorage) {
                if let Some(bitset) = Self::bitset(storage) {
                    if let Some(table_id) = storage.get_table_id_from_bitset(bitset) {
                        #[allow(non_snake_case)]
                        let ($($name,)*) = self;
                        ($(storage.insert_with_table_id(table_id, id, $name),)*);
                        return;
                    }
                }

                let mut registrator = storage.registrator();
                ($(registrator.register_inner::<$name>(0),)*);
                let table_id = registrator.finish();

                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                ($(storage.insert_with_table_id(table_id, id, $name),)*);
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
