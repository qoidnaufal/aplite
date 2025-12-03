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

    pub const fn new(index: u32, version: u32) -> Self {
        Self(index | version << 20)
    }

    pub const fn id(&self) -> EntityId {
        EntityId::new(self.0 & Self::INDEX_MASK)
    }

    pub const fn version(&self) -> EntityVersion {
        EntityVersion::new(self.0 >> 20 & Self::VERSION_MASK)
    }

    pub const fn index(&self) -> usize {
        self.id().index()
    }

    pub const fn raw(&self) -> u32 {
        self.0
    }
}

impl std::hash::Hash for EntityId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0 as _);
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

    pub fn create(&mut self) -> Entity {
        let id = self.recycled
            .pop()
            .unwrap_or_else(|| {
                let id = u32::try_from(self.versions.len())
                    .ok()
                    .expect("Created Entity should not exceed u32::MAX");

                debug_assert!(id <= Entity::INDEX_MASK);
                self.versions.push(0);
                id
            });
        Entity::new(id, self.versions[id as usize])
    }

    pub fn is_alive(&self, id: &Entity) -> bool {
        let version = self.versions[id.index()];
        version < Entity::VERSION_MASK && version == id.version().0
    }

    pub fn destroy(&mut self, id: Entity) {
        let idx = id.index();
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
