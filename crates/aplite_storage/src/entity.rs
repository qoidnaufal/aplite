// use std::sync::{OnceLock, RwLock};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(pub(crate) u32);

impl EntityId {
    #[inline(always)]
    pub(crate) const fn new(val: u32) -> Self {
        Self(val)
    }

    #[inline(always)]
    pub const fn index(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityVersion(pub(crate) u32);

impl EntityVersion {
    #[inline(always)]
    const fn new(val: u32) -> Self {
        Self(val)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entity(u32);

impl Entity {
    pub(crate) const INDEX_MASK: u32 = (1 << 20) - 1;
    pub(crate) const VERSION_MASK: u32 = (1 << 12) - 1;

    pub(crate) const fn with_id_version(index: u32, version: u32) -> Self {
        Self(index | version << 20)
    }

    // pub fn new() -> Self {
    //     let mut manager = ENTITY_MANAGER
    //         .get_or_init(EntityManager::init)
    //         .write()
    //         .unwrap();

    //     manager.create()
    // }

    // pub fn destroy(self) {
    //     let mut manager = ENTITY_MANAGER
    //         .get_or_init(EntityManager::init)
    //         .write()
    //         .unwrap();

    //     manager.destroy(self);
    // }

    // pub fn is_alive(&self) -> bool {
    //     ENTITY_MANAGER.get_or_init(EntityManager::init)
    //         .read()
    //         .unwrap()
    //         .is_alive(self)
    // }

    #[inline(always)]
    pub const fn id(&self) -> EntityId {
        EntityId::new(self.0 & Self::INDEX_MASK)
    }

    #[inline(always)]
    pub const fn version(&self) -> EntityVersion {
        EntityVersion::new(self.0 >> 20 & Self::VERSION_MASK)
    }

    #[inline(always)]
    pub const fn index(&self) -> usize {
        (self.0 & Self::INDEX_MASK) as usize
    }

    #[inline(always)]
    pub const fn raw(&self) -> u32 {
        self.0
    }
}

impl std::hash::Hash for EntityId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.0);
    }
}

impl std::hash::Hash for Entity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.id().0);
    }
}

impl std::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntityId({})", self.0)
    }
}

impl std::fmt::Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({})", self.raw())
    }
}

// static ENTITY_MANAGER: OnceLock<RwLock<EntityManager>> = OnceLock::new();

#[derive(Debug)]
pub struct EntityManager {
    versions: Vec<u32>,
    recycled: Vec<u32>,
}

impl Default for EntityManager {
    /// Create a new manager with no reserved capacity at all.
    /// If you want to preallocate a specific initial capacity, use [`IdManager::with_capacity`]
    fn default() -> Self {
        Self::new()
    }
}

impl EntityManager {
    /// Create a new manager with the specified capacity for the version manager & recycled,
    /// Using [`IdManager::default`] will create one with no preallocated capacity at all
    pub fn new() -> Self {
        Self {
            recycled: Vec::default(),
            versions: Vec::default(),
        }
    }

    // fn init() -> RwLock<Self> {
    //     RwLock::new(Self::new())
    // }

    pub fn create(&mut self) -> Entity {
        let id = self.recycled
            .pop()
            .unwrap_or_else(|| {
                let id = u32::try_from(self.versions.len())
                    .ok()
                    .expect("Created Entity should not exceed '(1 << 20) - 1'");

                debug_assert!(id <= Entity::INDEX_MASK);
                self.versions.push(0);
                id
            });

        Entity::with_id_version(id, self.versions[id as usize])
    }

    pub fn is_alive(&self, entity: &Entity) -> bool {
        let version = self.versions[entity.index()];
        version < Entity::VERSION_MASK && version == entity.version().0
    }

    pub fn destroy(&mut self, entity: Entity) {
        let idx = entity.index();
        if self.versions[idx] < u32::MAX {
            self.versions[idx] += 1;
            self.recycled.push(idx as u32);
        }
    }

    pub fn reset(&mut self) {
        self.recycled.clear();
        self.versions.clear();
    }
}
