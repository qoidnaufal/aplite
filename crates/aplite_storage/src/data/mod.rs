use std::ptr::NonNull;

use aplite_types::*;
use crate::entity::EntityId;

pub(crate) mod bitset;
pub(crate) mod component;
pub(crate) mod query;
pub(crate) mod table;

use bitset::Bitset;
use query::{Queryable, QueryData, QueryIter};
use table::{ComponentStorage, MarkedBuffer, TableId};
use component::{
    Component,
    ComponentEq,
    ComponentTuple,
    ComponentTupleExt,
    ComponentId,
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

macro_rules! component_bundle {
    ($($name:ident),*) => {
        impl<$($name: Component + 'static),*> ComponentTuple for ($($name,)*) {
            type Item = ($($name,)*);

            fn insert_bundle(self, entity: EntityId, storage: &mut ComponentStorage) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                if let Some(bitset) = storage.get_bitset::<Self>() {
                    ($(storage.insert(bitset, $name),)*);

                    let table_id = storage.table_ids[&bitset];
                    let table = &mut storage.tables[table_id.0];
                    table.indexes.set_index(entity.index(), table.entities.len());
                    table.entities.push(entity);

                    return;
                }

                let mut registrator = storage.archetype_builder();
                ($(registrator.register_component::<$name>(0),)*);

                let bitset = registrator.finish();

                ($(storage.insert(bitset, $name),)*);

                let table_id = storage.table_ids[&bitset];
                let table = &mut storage.tables[table_id.0];
                table.indexes.set_index(entity.index(), table.entities.len());
                table.entities.push(entity);
            }
        }

        impl<$($name: Component + 'static),*> ComponentTupleExt for ($($name,)*) {
            fn bitset(storage: &ComponentStorage) -> Option<Bitset> {
                let mut bitset = Bitset::new();
                ($(bitset.update(storage.get_component_id::<$name>()?.0),)*);
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

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(self, f)
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

macro_rules! query {
    ($($name:ident),*) => {
        impl<'a, $($name),*> QueryData<'a> for ($($name,)*)
        where
            $($name: Queryable<'a>),*,
            $($name::Item: Component + 'static),*,
        {
            type Items = ($(<$name as Queryable<'a>>::Item,)*);
            type RawBuffer = ($(NonNull<$name::Item>,)*);
            type Buffer = ($(Box<[MarkedBuffer<'a, $name>]>,)*);

            fn type_ids() -> Vec<std::any::TypeId> {
                let mut vec = vec![];
                ($(vec.push(std::any::TypeId::of::<<$name as Queryable>::Item>()),)*);
                vec
            }

            fn component_ids(source: &ComponentStorage) -> Option<Vec<ComponentId>> {
                let mut vec = vec![];
                ($(vec.push(source.get_component_id::<<$name as Queryable>::Item>()?),)*);
                Some(vec)
            }

            fn bitset(source: &ComponentStorage) -> Option<Bitset> {
                let mut bitset = Bitset::new();
                ($(bitset.update(source.get_component_id::<<$name as Queryable>::Item>()?.0),)*);
                Some(bitset)
            }

            fn get_buffer(source: &'a ComponentStorage, table_ids: &[&'a TableId]) -> Self::Buffer {
                ($(source.get_queryable_buffers_by_id::<$name>(table_ids),)*)
            }
        }

        impl<'a, $($name),*> Iterator for QueryIter<'a, ($($name,)*)>
        where
            $($name: Queryable<'a>),*,
            $($name::Item: Component + 'static),*,
        {
            type Item = ($($name,)*);

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(raws) = self.current {
                    #[allow(non_snake_case)]
                    let ($($name,)*) = raws;

                    if self.counter < self.len {
                        #[allow(non_snake_case)]
                        let ($($name,)*) = unsafe { ($($name.add(self.counter),)*) };
                        self.counter += 1;
                        return Some(($($name::convert($name.as_ptr()),)*));
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

macro_rules! query_one {
    ($name:ident) => {
        impl<'a, $name> QueryData<'a> for $name
        where
            $name: Queryable<'a>,
            $name::Item: 'static,
        {
            type Items = (<$name as Queryable<'a>>::Item,);
            type RawBuffer = NonNull<$name::Item>;
            type Buffer = Box<[MarkedBuffer<'a, $name>]>;

            fn type_ids() -> Vec<std::any::TypeId> {
                vec![std::any::TypeId::of::<<$name as Queryable>::Item>()]
            }

            fn component_ids(source: &ComponentStorage) -> Option<Vec<ComponentId>> {
                let component_id = source.get_component_id::<<$name as Queryable>::Item>()?;
                Some(vec![component_id])
            }

            fn bitset(source: &ComponentStorage) -> Option<Bitset> {
                let component_id = source.get_component_id::<<$name as Queryable>::Item>()?;
                Some(Bitset(1 << component_id.0))
            }

            fn get_buffer(source: &'a ComponentStorage, table_ids: &[&'a TableId]) -> Self::Buffer {
                source.get_queryable_buffers_by_id::<$name>(table_ids)
            }
        }

        impl<'a, $name> Iterator for QueryIter<'a, $name>
        where
            $name: Queryable<'a>,
            $name::Item: 'static,
        {
            type Item = $name;

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(raw) = self.current {
                    if self.counter < self.len {
                        let next = unsafe { raw.add(self.counter) };
                        self.counter += 1;
                        return Some($name::convert(next.as_ptr()));
                    }
                }

                let buffer = self.buffer.get_mut(self.buffer_counter)?;
                self.current = Some(buffer.start);
                self.buffer_counter += 1;
                self.counter = 0;
                self.len = buffer.len;
                self.next()
            }
        } 
    };
}

query_one!(A);
