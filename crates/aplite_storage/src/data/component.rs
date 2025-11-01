// use std::collections::HashMap;
// use std::any::{Any, TypeId};
use std::any::TypeId;

use std::slice::Iter;
use std::iter::Map;
use std::cell::UnsafeCell;

use super::table::Table;

use crate::entity::EntityId;
use crate::data::sparse_index::Index;

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
                let idx = source.ptr
                    .get_index(&id)?
                    .index();

                let removed = Some(($(
                    source.inner
                        .get_mut(&TypeId::of::<$name>())
                        .and_then(|any| any.downcast_mut::<Vec<UnsafeCell<$name>>>())
                        .map(|vec| vec.swap_remove(idx).into_inner())?,
                )*));

                let last = source.entities.last().unwrap();

                source.ptr.set_index(last, idx);
                source.ptr.set_null(&id);
                source.entities.swap_remove(idx);

                removed
            }
        }
    };
}

/// Query on many component type
pub struct Query<'a, Q: QueryData<'a>> {
    pub(crate) ptr: &'a [Index],
    pub(crate) inner: Option<Q::Data>,
}

impl<'a, Q: QueryData<'a>> Query<'a, Q> {
    pub fn new(source: &'a Table) -> Self {
        Self {
            ptr: &source.ptr.ptr,
            inner: Q::data(source),
        }
    }
}

pub struct QueryIter<'a, Q: QueryData<'a>> {
    pub(crate) inner: Option<Q::Iter>,
}

pub trait Queryable<'a> {
    type Item: 'static;
    type Output: 'a;

    /// Convert `UnsafeCell<T>` to `&T` or `&mut T`.
    fn convert(item: &UnsafeCell<Self::Item>) -> Self::Output;
}

impl<'a, T: 'static> Queryable<'a> for &'a T {
    type Item = T;
    type Output = &'a T;

    fn convert(item: &UnsafeCell<Self::Item>) -> Self::Output {
        unsafe { &*item.get() }
    }
}

impl<'a, T: 'static> Queryable<'a> for &'a mut T {
    type Item = T;
    type Output = &'a mut T;

    fn convert(item: &UnsafeCell<Self::Item>) -> Self::Output {
        unsafe { &mut *item.get() }
    }
}

pub trait QueryData<'a>: Sized {
    type Data;
    type Iter;

    fn data(source: &'a Table) -> Option<Self::Data>;
    fn query(source: &'a Table) -> Query<'a, Self>;
}

pub(crate) fn map_query<'a, Q: Queryable<'a>>(cell: &'a UnsafeCell<Q::Item>) -> Q::Output {
    Q::convert(cell)
}

pub(crate) type FnMapQuery<'a, Q> =
    fn(&'a UnsafeCell<<Q as Queryable<'a>>::Item>) -> <Q as Queryable<'a>>::Output;

macro_rules! query {
    ($($name:ident),*) => {
        impl<'a, $($name: Queryable<'a>),*> QueryData<'a> for ($($name,)*) {
            type Data = ($(&'a Vec<UnsafeCell<$name::Item>>,)*);
            type Iter = ($(Map<Iter<'a, UnsafeCell<$name::Item>>, FnMapQuery<'a, $name>>,)*);

            fn data(source: &'a Table) -> Option<Self::Data> {
                Some(($(
                    source.inner
                        .get(&TypeId::of::<$name::Item>())
                        .and_then(|any| any.downcast_ref::<Vec<UnsafeCell<$name::Item>>>())?,
                )*))
            }

            fn query(source: &'a Table) -> Query<'a, Self> {
                Query::new(source)
            }
        }

        impl<'a, $($name: Queryable<'a>),*> Query<'a, ($($name,)*)> {
            pub fn iter(&self) -> QueryIter<'a, ($($name,)*)> {
                #[allow(non_snake_case)]
                let Some(($($name,)*)) = self.inner else {
                    return QueryIter { inner: None }
                };

                let inner = Some(($($name.iter().map(map_query::<'a, $name> as FnMapQuery<'a, $name>),)*));

                QueryIter { inner }
            }

            pub fn get(&self, id: &EntityId) -> Option<($($name::Output,)*)> {
                #[allow(non_snake_case)]
                let Some(($($name,)*)) = self.inner else {
                    return None
                };

                let index = self.ptr
                    .get(id.index())
                    .and_then(|i| (!i.is_null()).then_some(i.index()))?;

                Some(($($name::convert(&$name[index]),)*))
            }
        }

        impl<'a, $($name: Queryable<'a>),*> IntoIterator for &'a Query<'a, ($($name,)*)> {
            type IntoIter = QueryIter<'a, ($($name,)*)>;
            type Item = ($($name::Output,)*);

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }

        impl<'a, $($name: Queryable<'a>),*> IntoIterator for Query<'a, ($($name,)*)> {
            type IntoIter = QueryIter<'a, ($($name,)*)>;
            type Item = ($($name::Output,)*);

            fn into_iter(self) -> Self::IntoIter {
                #[allow(non_snake_case)]
                let Some(($($name,)*)) = self.inner else {
                    return QueryIter { inner: None }
                };

                let inner = Some(($($name.iter().map(map_query::<'a, $name> as FnMapQuery<'a, $name>),)*));

                QueryIter { inner }
            }
        }

        impl<'a, $($name: Queryable<'a>),*> Iterator for QueryIter<'a, ($($name,)*)> {
            type Item = ($($name::Output,)*);

            fn next(&mut self) -> Option<Self::Item> {
                #[allow(non_snake_case)]
                let Some(($($name,)*)) = self.inner.as_mut() else { return None };
                Some(($($name.next()?,)*))
            }
        }
    };
}

macro_rules! query_one {
    ($name:ident) => {
        impl<'a, $name: Queryable<'a>> QueryData<'a> for $name {
            type Data = &'a Vec<UnsafeCell<$name::Item>>;
            type Iter = Map<Iter<'a, UnsafeCell<$name::Item>>, FnMapQuery<'a, $name>>;

            fn data(source: &'a Table) -> Option<Self::Data> {
                Some(
                    source.inner
                        .get(&TypeId::of::<$name::Item>())
                        .and_then(|any| any.downcast_ref::<Vec<UnsafeCell<$name::Item>>>())?
                )
            }

            fn query(source: &'a Table) -> Query<'a, Self> {
                Query::new(source)
            }
        }

        impl<'a, $name: Queryable<'a>> Query<'a, $name> {
            pub fn iter(&self) -> QueryIter<'a, $name> {
                #[allow(non_snake_case)]
                let Some($name) = self.inner else {
                    return QueryIter { inner: None }
                };

                let inner = Some($name.iter().map(map_query::<'a, $name> as FnMapQuery<'a, $name>));

                QueryIter { inner }
            }

            pub fn get(&self, id: &EntityId) -> Option<$name::Output> {
                let data = self.inner?;

                let index = self.ptr
                    .get(id.index())
                    .and_then(|i| (!i.is_null()).then_some(i.index()))?;

                Some($name::convert(&data[index]))
            }
        }

        impl<'a, $name: Queryable<'a>> IntoIterator for &'a Query<'a, $name> {
            type IntoIter = QueryIter<'a, $name>;
            type Item = $name::Output;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }

        impl<'a, $name: Queryable<'a>> IntoIterator for Query<'a, $name> {
            type IntoIter = QueryIter<'a, $name>;
            type Item = $name::Output;

            fn into_iter(self) -> Self::IntoIter {
                #[allow(non_snake_case)]
                let Some($name) = self.inner else {
                    return QueryIter { inner: None }
                };

                let inner = Some($name.iter().map(map_query::<'a, $name> as FnMapQuery<'a, $name>));

                QueryIter { inner }
            }
        }

        impl<'a, $name: Queryable<'a>> Iterator for QueryIter<'a, $name> {
            type Item = $name::Output;

            fn next(&mut self) -> Option<Self::Item> {
                #[allow(non_snake_case)]
                let Some($name) = self.inner.as_mut() else { return None };
                Some($name.next()?)
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
