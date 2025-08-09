/// A macro to conveniently implement [`Entity`] trait to be stored in the [`Tree`] or [`IndexMap`](crate::index_map::IndexMap).
/// You just need to specify the name.
/// # Example
/// ```ignore
/// entity! {
///     SuperUniqueIdName,
///     AnotherId,
/// }
///
/// let mut tree: Tree<SuperUniqueIdName> = Tree::new();
/// let super_unique_id_name: SuperUniqueIdName = tree.create_entity();
/// let another_id = AnotherId::new();
/// ```

#[macro_export]
macro_rules! entity {
    { $vis:vis $name:ident } => {
        use aplite_storage::Entity;

        #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        $vis struct $name(u64);

        impl Entity for $name {
            // not sure if this is a good idea to do this
            fn new(index: u32, version: u32) -> Self {
                Self(((version as u64) << 32) + index as u64)
            }

            fn index(&self) -> usize {
                (self.0 as u32) as usize
            }

            fn version(&self) -> u32 {
                (self.0 >> 32) as u32
            }

            fn hasher(&self) -> u64 {
                self.0
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.index())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.index())
            }
        }

        impl std::hash::Hash for $name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                state.write_u64(self.hasher());
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
