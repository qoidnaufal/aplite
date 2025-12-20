use std::any::TypeId;

use crate::data::table::ComponentStorage;
use crate::entity::Entity;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentId(pub(crate) u64);

impl ComponentId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id as _)
    }
}

impl std::fmt::Debug for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ComponentId({})", self.0)
    }
}

impl std::hash::Hash for ComponentId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentBitset(pub(crate) u64);

impl ComponentBitset {
    pub(crate) fn new() -> Self {
        Self(0)
    }

    pub(crate) fn update(&mut self, component_id: ComponentId) {
        self.0 |= 1 << component_id.0
    }

    pub(crate) fn contains(&self, component_id: ComponentId) -> bool {
        self.0 & 1 << component_id.0 == 1 << component_id.0
    }
}

impl std::hash::Hash for ComponentBitset {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::fmt::Debug for ComponentBitset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ComponentBitSet({:b})", self.0)
    }
}

pub trait Component: Sized + 'static {
    fn type_id() -> TypeId {
        TypeId::of::<Self>()
    }
}

// impl<T: Sized + 'static> Component for T {}

pub trait ComponentTuple {
    type Item;

    fn insert_bundle(self, entity: Entity, storage: &mut ComponentStorage);
    // fn for_each(&self, f: impl FnMut(Self::Item));
}

pub(crate) trait ComponentTupleExt {
    fn bitset(storage: &ComponentStorage) -> Option<ComponentBitset>;
}

// pub trait IntoComponent: Sized + 'static {
//     type Item: Component;

//     fn into_component(self) -> Self::Item;
// }

#[macro_export]
macro_rules! make_component {
    ($vis:vis struct $name:ident($($num:tt $ty:ty),*)) => {
        $vis struct $name($($ty),*);

        impl Component for $name {}

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut dbt = f.debug_tuple(stringify!($name));
                $(dbt.field(&self.$num);)*
                dbt.finish()
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
}
