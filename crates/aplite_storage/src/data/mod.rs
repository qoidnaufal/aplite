use crate::entity::Entity;

pub(crate) mod component;
pub(crate) mod query;
pub(crate) mod table;

use component::{Component, ComponentTuple, ComponentTupleExt, ComponentBitset};
use query::{Queryable, QueryData};
use table::ComponentStorage;

macro_rules! component_bundle {
    ($($name:ident),*) => {
        impl<$($name: Component),*> ComponentTuple for ($($name,)*) {
            type Item = ($($name,)*);

            fn insert_bundle(self, entity: Entity, storage: &mut ComponentStorage) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                if let Some(table_id) = storage.get_table_id_from_bundle::<Self>() {
                    ($(storage.insert_with_table_id(table_id, $name),)*);

                    let table = storage.get_table_mut_from_table_id(table_id);
                    table.indexes.set_index(entity.index(), table.entities.len());
                    table.entities.push(entity);

                    return;
                }

                let mut registrator = storage.registrator();
                ($(registrator.register_component::<$name>(0),)*);

                let table_id = registrator.finish();
                ($(storage.insert_with_table_id(table_id, $name),)*);

                let table = storage.get_table_mut_from_table_id(table_id);
                table.indexes.set_index(entity.index(), table.entities.len());
                table.entities.push(entity);
            }
        }

        impl<$($name: Component),*> ComponentTupleExt for ($($name,)*) {
            fn bitset(storage: &ComponentStorage) -> Option<ComponentBitset> {
                let mut bitset = ComponentBitset::new();
                ($(bitset.update(storage.get_component_id::<$name>()?),)*);
                Some(bitset)
            }
        }
    };
}

// macro_rules! query {
//     ($($name:ident),*) => {
//         impl<'a, $($name: Queryable<'a>),*> QueryData<'a> for ($($name,)*) {}
//     };
// }

// macro_rules! query_one {
//     ($name:ident) => {
//         impl<'a, $name: Queryable<'a>> QueryData<'a> for $name {}
//     };
// }

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

// impl_tuple_macro!(
//     query,
//     A, B, C, D, E,
//     F, G, H, I, J,
//     K, L, M, N, O,
//     P, Q, R, S, T,
//     U, V, W, X, Y,
//     Z
// );

// query_one!(A);

#[macro_export]
macro_rules! make_component {
    ($vis:vis struct $name:ident($ty:ty)) => {
        $vis struct $name($ty);

        impl Component for $name {}

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.0)
                    .finish()
            }
        }
    };

    ($vis:vis struct $name:ident { $($field:ident: $ty:ty),* }) => {
        $vis struct $name { $($field: $ty),* }

        impl Component for $name {}

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut dbs = f.debug_struct(stringify!($name));
                $(dbs.field(stringify!($field), &self.$field);)*
                dbs.finish()
            }
        }
    };

    // ($vis:vis struct $name:ident<$($gen:ty),?> { $($field:ident: $ty:ty),? }) => {
    //     $vis struct $name { $($field: $ty),* }

    //     impl Component for $name {}

    //     impl std::fmt::Debug for $name {
    //         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //             let mut dbs = f.debug_struct(stringify!($name));
    //             $(dbs.field(stringify!($field), &self.$field);)*
    //             dbs.finish()
    //         }
    //     }
    // };
}
