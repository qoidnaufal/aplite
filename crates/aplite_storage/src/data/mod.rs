use aplite_types::*;
use crate::entity::Entity;

pub(crate) mod component;
pub(crate) mod query;
pub(crate) mod table;

use component::{Component, ComponentEq, ComponentTuple, ComponentTupleExt, ComponentBitset};
// use query::{Queryable, QueryData};
use table::ComponentStorage;

macro_rules! impl_tuple_macro {
    ($macro:ident, $next:tt) => {
        $macro!{$next}
    };
    ($macro:ident, $next:tt, $($rest:tt),*) => {
        $macro!{$next, $($rest),*}
        impl_tuple_macro!{$macro, $($rest),*}
    };
}

macro_rules! component_bundle {
    ($($name:ident),*) => {
        impl<$($name: Component + 'static),*> ComponentTuple for ($($name,)*) {
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

            // fn for_each(&self, mut f: impl FnMut(&dyn Component)) {
            //     #[allow(non_snake_case)]
            //     let ($($name,)*) = self;

            //     ($(f($name),)*);
            // }
        }

        impl<$($name: Component + 'static),*> ComponentTupleExt for ($($name,)*) {
            fn bitset(storage: &ComponentStorage) -> Option<ComponentBitset> {
                let mut bitset = ComponentBitset::new();
                ($(bitset.update(storage.get_component_id::<$name>()?),)*);
                Some(bitset)
            }
        }
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

macro_rules! partial_eq {
    ($(($name:ident, $other:ident)),*) => {
        impl<$($name: Component + PartialEq + 'static),*> ComponentEq for ($($name,)*) {
            fn component_eq(&self, other: &Self) -> bool {
                let mut res = true;

                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                #[allow(non_snake_case)]
                let ($($other,)*) = other;

                ($(res &= $name.eq($other),)*);
                res
            }
        }
    };
}

impl_tuple_macro!(
    partial_eq,
    (A, AA), (B, BB), (C, CC), (D, DD), (E, EE),
    (F, FF), (G, GG), (H, HH), (I, II), (J, JJ),
    (K, KK), (L, LL), (M, MM), (N, NN), (O, OO),
    (P, PP), (Q, QQ), (R, RR), (S, SS), (T, TT),
    (U, UU), (V, VV), (W, WW), (X, XX), (Y, YY),
    (Z, ZZ)
);

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
    ($name:ident) => {
        impl Component for $name {}
    };

    ($vis:vis struct $name:ident($ty:ty)) => {
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
}

make_component!(Length);
make_component!(Vec2f);
make_component!(Vec2u);
make_component!(Point);
make_component!(Size);
make_component!(Rect);

make_component!(CornerRadius);
make_component!(Fraction);

make_component!(Rgba);
make_component!(Paint);
make_component!(ImageData);
make_component!(Matrix3x2);
