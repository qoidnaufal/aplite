use std::ptr::NonNull;

use aplite_types::*;
use aplite_bitset::Bitset;
use crate::entity::EntityId;

pub(crate) mod archetype;
pub(crate) mod component;
pub(crate) mod component_storage;
pub(crate) mod query;
pub(crate) mod table;

use query::{QueryData, QueryIter};
use component_storage::{ComponentStorage, MarkedBuffer};
use component::{
    Component,
    ComponentEq,
};

macro_rules! impl_tuple_macro {
    ($macro:ident, $next:tt) => {
        $macro!{$next}
    };
    ($macro:ident, $next:tt, $($rest:tt),*) => {
        $macro!{$next, $($rest),*}
        impl_tuple_macro!{$macro, $($rest),*}
    };
}

macro_rules! component_tuple {
    ($($name:ident),*) => {
        impl<$($name: Component + 'static),*> Component for ($($name,)*) {
            type Item = ($($name,)*);

            fn insert(self, entity: EntityId, storage: &mut ComponentStorage) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                fn get_component_id<$($name: Component + 'static),*>(
                    storage: &mut ComponentStorage
                ) -> Option<Bitset>
                {
                    let mut bitset = Bitset::default();
                    ($(bitset.add_bit(storage.get_component_id::<$name>()?.0),)*);
                    Some(bitset)
                }

                if let Some(bitset) = get_component_id::<$($name),*>(storage) {
                    let id = storage.archetype_ids[&bitset];

                    ($(storage.insert_archetype_by_id(id, $name),)*);

                    let table = &mut storage.archetype_tables[id.0];
                    table.indexes.set_index(entity.index(), table.entities.len());
                    table.entities.push(entity);

                    return;
                }

                let mut builder = storage.archetype_builder();
                ($(builder.register_component::<$name>(0),)*);

                let id = builder.finish();

                ($(storage.insert_archetype_by_id(id, $name),)*);

                let table = &mut storage.archetype_tables[id.0];
                table.indexes.set_index(entity.index(), table.entities.len());
                table.entities.push(entity);
            }
        }
    };
}

impl_tuple_macro!(
    component_tuple,
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

#[macro_export]
macro_rules! make_component {
    ($name:ident) => {
        impl Component for $name {
            type Item = $name;

            fn insert(self, entity: EntityId, storage: &mut ComponentStorage) {
                if let Some(component_id) = storage.get_component_id::<$name>() {
                    let component_id_bitset = Bitset::new(component_id.index());
                }
            }
        }
    };

    ($vis:vis struct $name:ident($ty:ty)) => {
        #[derive(PartialEq)]
        $vis struct $name($ty);

        impl Component for $name {
            type Item = $name;

            fn insert(self, entity: EntityId, storage: &mut ComponentStorage) {}
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.0)
                    .finish()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(self, f)
            }
        }
    };

    ($vis:vis struct $name:ident { $($field:ident: $ty:ty),* }) => {
        #[derive(PartialEq)]
        $vis struct $name { $($field: $ty),* }

        impl Component for $name {
            type Item = $name;

            fn insert(self, entity: EntityId, storage: &mut ComponentStorage) {}
        }

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

macro_rules! query {
    ($($name:ident),*) => {
        impl<'a, $($name),*> QueryData for ($($name,)*)
        where
            $($name: QueryData<Fetch<'a> = $name, State = NonNull<<$name as QueryData>::Item>>),*,
            $($name::Item: Component + 'static),*,
        {
            type Item = ($(<$name as QueryData>::Item,)*);
            type Fetch<'b> = ($($name,)*);
            type State = ($(NonNull<<$name as QueryData>::Item>,)*);
            type Buffer<'b> = ($(Box<[MarkedBuffer<'b, $name>]>,)*);

            fn matched_component_ids(source: &ComponentStorage) -> Option<Bitset> {
                let mut bitset = Bitset::default();
                ($(
                    bitset.add_bit(
                        source.get_component_id::<<$name as QueryData>::Item>()
                            .map(|id| id.0)?
                    ),
                )*);
                Some(bitset)
            }

            fn get<'b>(input: Self::State) -> Self::Fetch<'b> {
                #[allow(non_snake_case)]
                let ($($name,)*) = input;
                ($($name::get($name),)*)
            }

            fn get_buffer<'b>(source: &'b ComponentStorage, table_ids: Bitset) -> Self::Buffer<'b> {
                ($(source.get_marked_buffers::<$name>(table_ids),)*)
            }
        }

        impl<'a, $($name),*> Iterator for QueryIter<'a, ($($name,)*)>
        where
            $($name: QueryData<Fetch<'a> = $name, State = NonNull<<$name as QueryData>::Item>>),*,
            $($name::Item: Component + 'static),*,
        {
            type Item = ($(<$name as QueryData>::Fetch<'a>,)*);

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(raws) = self.current {
                    #[allow(non_snake_case)]
                    let ($($name,)*) = raws;

                    if self.counter < self.len {
                        #[allow(non_snake_case)]
                        let ($($name,)*) = unsafe { ($($name.add(self.counter),)*) };
                        self.counter += 1;
                        return Some(($($name::get($name),)*));
                    }
                }

                #[allow(non_snake_case)]
                let ($($name,)*): &mut ($(Box<[MarkedBuffer<'a, $name>]>,)*) = &mut self.buffer;

                #[allow(non_snake_case)]
                let ($($name,)*) = ($($name.get_mut(self.buffer_counter)?,)*);

                self.current = Some(($($name.start,)*));
                ($(self.len = $name.len,)*);
                self.buffer_counter += 1;
                self.counter = 0;
                self.next()
            }
        } 
    };
}

impl_tuple_macro!(
    query,
    A, B, C, D, E,
    F, G, H, I, J,
    K, L, M, N, O,
    P, Q, R, S, T,
    U, V, W, X, Y,
    Z
);
