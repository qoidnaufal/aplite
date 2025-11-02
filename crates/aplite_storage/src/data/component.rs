// use std::collections::HashMap;
// use std::any::{Any, TypeId};
use std::any::TypeId;

use std::cell::UnsafeCell;

use super::table::Table;

use crate::entity::EntityId;

// pub struct ComponentStorage<E: Entity + 'static> {
//     pub(crate) data: HashMap<TypeId, Box<dyn Any>>,
//     pub(crate) tables: HashMap<TableId, Table<E>>,
// }

pub trait Component: Sized + 'static {
    type Item;

    fn component_id() -> Vec<TypeId>;

    /// Register value(s) to the table
    fn register(self, id: &EntityId, table: &mut Table);

    /// Use this function carefully, or else this will mess up your data
    fn remove(id: EntityId, source: &mut Table) -> Option<Self::Item>;
}

macro_rules! component {
    ($($name:ident),*) => {
        impl<$($name: 'static),*> Component for ($($name,)*) {
            type Item = ($($name,)*);

            fn component_id() -> Vec<TypeId> {
                let mut vec = vec![];
                $(vec.push(TypeId::of::<$name>());)*
                vec
            }

            fn register(self, id: &EntityId, table: &mut Table) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                $(table.insert(id, $name);)*
            }

            fn remove(id: EntityId, source: &mut Table) -> Option<Self::Item> {
                let idx = source.indexes
                    .get_index(&id)?
                    .index();

                let removed = Some(($(
                    source.inner
                        .get_mut(&TypeId::of::<$name>())
                        .and_then(|any| any.downcast_mut::<Vec<UnsafeCell<$name>>>())
                        .map(|vec| vec.swap_remove(idx).into_inner())?,
                )*));

                let last = source.entities.last().unwrap();

                source.indexes.set_index(last, idx);
                source.indexes.set_null(&id);
                source.entities.swap_remove(idx);

                removed
            }
        }
    };
}

use crate::impl_tuple_macro;

impl_tuple_macro!(
    component,
    A, B, C, D, E,
    F, G, H, I, J
    // K, L, M, N, O,
    // P, Q, R, S, T,
    // U, V, W, X, Y,
    // Z
);

// #[derive(Debug)]
// pub struct InvalidComponent(&'static str);

// impl std::error::Error for InvalidComponent {}

// impl std::fmt::Display for InvalidComponent {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Invalid component {}", self.0)
//     }
// }

// impl PartialEq for InvalidComponent {
//     fn eq(&self, other: &Self) -> bool {
//         self.0 == other.0
//     }
// }

// impl Eq for InvalidComponent {}

pub trait IntoComponent: Sized + 'static {
    type Item: Component;
    fn into_component(self) -> Self::Item;

    fn component_id() -> Vec<TypeId> {
        <<Self as IntoComponent>::Item as Component>::component_id()
    }
}

impl<T: Component> IntoComponent for T {
    type Item = Self;
    fn into_component(self) -> Self::Item {
        self
    }
}
