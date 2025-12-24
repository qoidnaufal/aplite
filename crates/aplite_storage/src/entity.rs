// use std::sync::{OnceLock, RwLock};

/*
#########################################################
#
# EntityId
#
#########################################################
*/

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(pub(crate) u32);

impl EntityId {
    #[inline(always)]
    pub(crate) const fn new(val: u32) -> Self {
        Self(val)
    }
    #[inline(always)]
    pub const fn from_usize(val: usize) -> Self {
        Self(val as _)
    }

    #[inline(always)]
    pub const fn index(&self) -> usize {
        self.0 as usize
    }
}

/*
#########################################################
#
# EntityVersion
#
#########################################################
*/

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityVersion(pub(crate) u32);

impl EntityVersion {
    #[inline(always)]
    const fn new(val: u32) -> Self {
        Self(val)
    }
}

/*
#########################################################
#
# Entity
#
#########################################################
*/

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entity(u64);

impl Entity {
    pub(crate) const MASK32: u32 = u32::MAX;
    pub(crate) const MASK64: u64 = (1 << 32) - 1;

    pub(crate) const fn with_id_version(index: u32, version: u32) -> Self {
        Self(index as u64 | (version as u64) << 32)
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
        EntityId::new((self.0 as u32) & Self::MASK32)
    }

    #[inline(always)]
    pub const fn version(&self) -> EntityVersion {
        EntityVersion::new((self.0 >> 32 & Self::MASK64) as u32)
    }

    #[inline(always)]
    pub const fn index(&self) -> usize {
        (self.0 & Self::MASK64) as usize
    }

    #[inline(always)]
    pub const fn raw(&self) -> u64 {
        self.0
    }
}

impl std::hash::Hash for EntityId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::hash::Hash for Entity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
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

/*
#########################################################
#
# EntityManager
#
#########################################################
*/

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

                debug_assert!(id <= Entity::MASK32);
                self.versions.push(0);
                id
            });

        Entity::with_id_version(id, self.versions[id as usize])
    }

    pub fn is_alive(&self, entity: &Entity) -> bool {
        let version = self.versions[entity.index()];
        version < Entity::MASK32 && version == entity.version().0
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
