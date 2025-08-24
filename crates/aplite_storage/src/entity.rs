/// A trait that needs to be implemented for any type to be stored in the [`Tree`](crate::tree::Tree) or [`IndexMap`](crate::index_map::IndexMap)
pub trait Entity
where
    Self : std::fmt::Debug + Copy + PartialEq + PartialOrd
{
    /// If you created this manually, you also need to manually [`insert()`](crate::tree::Tree::insert) it to the [`Tree`](crate::tree::Tree).
    /// The [`Tree`](crate::tree::Tree) provides a hassle free [`create_entity()`](Tree::create_entity) method
    /// to create an [`Entity`] and automatically insert it.
    fn new(index: u32, version: u32) -> Self;

    /// The index where this [`Entity`] is being stored inside the [`Tree`]
    fn index(&self) -> usize;

    /// The version of this [`Entity`]
    fn version(&self) -> u32;
}

#[macro_export]
/// A macro to conveniently implement [`Entity`] trait.
/// # Usage
/// ```ignore
/// entity! { pub(crate) UniqueId }
///
/// struct MyData {}
///
/// struct MyStorage {
///     inner: IndexMap<UniqueId, MyData>
/// }
///
/// let mut storage = MyStorage::new();
/// let data = MyData {};
/// let id = storage.inner.insert(data);
/// ```
macro_rules! entity {
    { $vis:vis $name:ident } => {
        #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        $vis struct $name(u32, u32);

        impl $name {
            $vis fn index(&self) -> usize {
                self.0 as usize
            }
        }

        impl Entity for $name {
            fn new(index: u32, version: u32) -> Self {
                Self(index, version)
            }

            fn index(&self) -> usize {
                self.index()
            }

            fn version(&self) -> u32 {
                self.1
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl std::hash::Hash for $name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                state.write_u64(self.0 as u64);
            }
        }

        impl PartialEq<&Self> for $name {
            fn eq(&self, other: &&Self) -> bool {
                self.0 == other.0 && self.1 == other.1
            }
        }
    };

    { $vis:vis $name:ident, } => {
        entity! { $vis $name }
    };

    { $vis:vis $name:ident, $($vis2:vis $name2:ident),* } => {
        entity! { $vis $name }
        entity! { $($vis2 $name2),* }
    };

    { $vis:vis $name:ident, $($vis2:vis $name2:ident),*, } => {
        entity! { $vis $name }
        entity! { $($vis2 $name2),* }
    };
}
