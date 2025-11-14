use super::table::ComponentTable;

use crate::entity::Entity;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ComponentId(pub(crate) u64);

impl std::hash::Hash for ComponentId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// There can only be 64 different components with this current implementation
pub(crate) struct ComponentBitSet(pub(crate) u64);

impl std::hash::Hash for ComponentBitSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::fmt::Debug for ComponentBitSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ComponentBitSet({:b})", self.0)
    }
}

pub trait Component: Sized + 'static {}

pub trait ComponentBundle {
    type Item;

    fn insert_bundle(self, id: &Entity, table: &mut ComponentTable);
}

macro_rules! component_bundle {
    ($($name:ident),*) => {
        impl<$($name: Component),*> ComponentBundle for ($($name,)*) {
            type Item = ($($name,)*);

            fn insert_bundle(self, id: &Entity, table: &mut ComponentTable) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                ($(table.insert(id, $name),)*);
            }
        }
    };
}

use crate::impl_tuple_macro;

impl_tuple_macro!(
    component_bundle,
    A, B, C, D, E,
    F, G, H, I, J
    // K, L, M, N, O,
    // P, Q, R, S, T,
    // U, V, W, X, Y,
    // Z
);

pub trait IntoComponent: Sized + 'static {
    type Item: Component;

    fn into_component(self) -> Self::Item;
}

// impl<T: Component> IntoComponent for T {
//     type Item = Self;

//     fn into_component(self) -> Self::Item {
//         self
//     }
// }
