use std::any::TypeId;
use std::slice::Iter;
use std::iter::Map;
use std::cell::UnsafeCell;

use super::table::Table;

use crate::entity::Entity;
use crate::data::sparse_index::Index;

pub trait Component: Sized + 'static {
    type Item;

    /// Register value(s) to the table
    fn register<E: Entity + 'static>(self, entity: &E, table: &mut Table<E>);

    /// Use this function carefully, or else this will mess up your data
    fn remove<E: Entity + 'static>(entity: E, source: &mut Table<E>) -> Option<Self::Item>;
}

macro_rules! component {
    ($($name:ident),*) => {
        impl<$($name: 'static),*> Component for ($($name,)*) {
            type Item = ($($name,)*);

            fn register<En: Entity + 'static>(self, entity: &En, table: &mut Table<En>) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                $(table.insert(entity, $name);)*
            }

            fn remove<En: Entity + 'static>(entity: En, source: &mut Table<En>) -> Option<Self::Item> {
                let idx = source.ptr
                    .get_index(&entity)?
                    .index();

                let removed = Some(($(
                    source.inner
                        .get_mut(&TypeId::of::<$name>())
                        .and_then(|any| any.downcast_mut::<Vec<UnsafeCell<$name>>>())
                        .map(|vec| vec.swap_remove(idx).into_inner())?,
                )*));

                let last = source.entities.last().unwrap();

                source.ptr.set_index(last, idx);
                source.ptr.set_null(&entity);
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
    pub fn new<E: Entity>(source: &'a Table<E>) -> Self {
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
    fn convert(fetch: &UnsafeCell<Self::Item>) -> Self::Output;
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

    fn data<E: Entity + 'static>(source: &'a Table<E>) -> Option<Self::Data>;
    fn query<E: Entity + 'static>(source: &'a Table<E>) -> Query<'a, Self>;
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

            fn data<En: Entity + 'static>(source: &'a Table<En>) -> Option<Self::Data> {
                Some(($(
                    source.inner
                        .get(&TypeId::of::<$name::Item>())
                        .and_then(|any| any.downcast_ref::<Vec<UnsafeCell<$name::Item>>>())?,
                )*))
            }

            fn query<En: Entity + 'static>(source: &'a Table<En>) -> Query<'a, Self> {
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

            pub fn get<En: Entity>(&self, entity: &En) -> Option<($($name::Output,)*)> {
                #[allow(non_snake_case)]
                let Some(($($name,)*)) = self.inner else {
                    return None
                };

                let index = self.ptr
                    .get(entity.index())
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

            fn data<En: Entity + 'static>(source: &'a Table<En>) -> Option<Self::Data> {
                Some(
                    source.inner
                        .get(&TypeId::of::<$name::Item>())
                        .and_then(|any| any.downcast_ref::<Vec<UnsafeCell<$name::Item>>>())?
                )
            }

            fn query<En: Entity + 'static>(source: &'a Table<En>) -> Query<'a, Self> {
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

            pub fn get<En: Entity>(&self, entity: &En) -> Option<$name::Output> {
                #[allow(non_snake_case)]
                let Some($name) = self.inner else {
                    return None
                };

                let index = self.ptr
                    .get(entity.index())
                    .and_then(|i| (!i.is_null()).then_some(i.index()))?;

                Some($name::convert(&$name[index]))
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
}

impl<T: Component> IntoComponent for T {
    type Item = Self;
    fn into_component(self) -> Self::Item {
        self
    }
}
