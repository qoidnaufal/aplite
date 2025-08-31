/// A trait that needs to be implemented for any type to be stored in the [`Tree`](crate::tree::Tree), [`IndexMap`](crate::index_map::IndexMap) or [`DataStore`](crate::data_store::DataStore)
pub trait Entity where Self : std::fmt::Debug + Sized + Copy + PartialEq + PartialOrd
{
    const VERSION_BITS: u8 = 10;
    const VERSION_MASK: u16 = (1 << Self::VERSION_BITS) - 1;
    const INDEX_BITS: u8 = 22;
    const INDEX_MASK: u32 = (1 << Self::INDEX_BITS) - 1;

    /// Threre's no need to do this manually, the entity creation is integrated with [`EntityManager`] or [`IndexMap`](crate::index_map::IndexMap)
    fn new(index: u32, version: u16) -> Self;

    /// The index where this [`Entity`] is being stored inside the [`Tree`]
    fn index(&self) -> usize;

    /// The version of this [`Entity`]
    fn version(&self) -> u16;
}

/// A macro to conveniently implement [`Entity`] trait.
/// # Usage
/// ```ignore
/// create_entity! { pub(crate) UniqueId }
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
#[macro_export]
macro_rules! create_entity {
    { $vis:vis $name:ident } => {
        #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        $vis struct $name(u32);

        impl Entity for $name {
            fn new(index: u32, version: u16) -> Self {
                Self((version as u32) << Self::INDEX_BITS | index)
            }

            fn index(&self) -> usize {
                (self.0 & Self::INDEX_MASK) as usize
            }

            fn version(&self) -> u16 {
                ((self.0 >> Self::INDEX_BITS) as u16) & Self::VERSION_MASK
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
                self.0.hash(state)
            }
        }

        impl PartialEq<&Self> for $name {
            fn eq(&self, other: &&Self) -> bool {
                self.0 == other.0
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

#[derive(Debug)]
pub struct EntityManager<E: Entity> {
    recycled: std::collections::VecDeque<u32>,
    version_manager: Vec<u16>,
    marker: std::marker::PhantomData<E>,
}

impl<E: Entity> Default for EntityManager<E> {
    /// Create a new manager with no preallocated capacity at all.
    /// If you want to preallocate a specific initial capacity, use [`EntityManager::with_version_capacity`] or [`EntityManager::with_same_capacity`]
    fn default() -> Self {
        Self {
            recycled: std::collections::VecDeque::default(),
            version_manager: Vec::default(),
            marker: std::marker::PhantomData,
        }
    }
}

impl<E: Entity> EntityManager<E> {
    const MINIMUM_FREE_INDEX: usize = 1 << E::VERSION_BITS;

    /// Create a new manager with the specified capacity for the version manager,
    /// and the recycled capacity will be set to 1 << [`Entity::VERSION_BITS`].
    /// Using [`EntityManager::default`] will create one with no preallocated capacity at all
    pub fn with_version_capacity(capacity: usize) -> Self {
        Self {
            recycled: std::collections::VecDeque::with_capacity(Self::MINIMUM_FREE_INDEX),
            version_manager: Vec::with_capacity(capacity),
            marker: std::marker::PhantomData,
        }
    }

    /// Create a new manager with the specified capacity for the version manager & recycled,
    /// Using [`EntityManager::default`] will create one with no preallocated capacity at all
    pub fn with_same_capacity(capacity: usize) -> Self {
        Self {
            recycled: std::collections::VecDeque::with_capacity(capacity),
            version_manager: Vec::with_capacity(capacity),
            marker: std::marker::PhantomData,
        }
    }

    pub fn create(&mut self) -> E {
        let id = if self.recycled.len() >= (Self::MINIMUM_FREE_INDEX) {
            self.recycled.pop_front().unwrap()
        } else {
            let id = self.version_manager.len() as u32;
            assert!(id <= E::INDEX_MASK);
            self.version_manager.push(0);
            id
        };

        E::new(id, self.version_manager[id as usize])
    }

    pub fn is_alive(&self, e: &E) -> bool {
        self.version_manager[e.index()] == e.version()
    }

    pub fn destroy(&mut self, e: E) {
        let idx = e.index();
        self.version_manager[idx] += 1;
        self.recycled.push_back(idx as u32);
    }

    pub fn reset(&mut self) {
        self.recycled.clear();
        self.version_manager.clear();
    }
}
