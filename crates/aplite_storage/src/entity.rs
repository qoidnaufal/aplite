#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entity {
    pub(crate) index: u32,
    pub(crate) version: u32,
}

impl Entity {
    pub const fn new(index: u32, version: u32) -> Self {
        Self {
            index,
            version,
        }
    }

    pub fn index(&self) -> usize {
        self.index as usize
    }

    pub const fn version(&self) -> u32 {
        self.version
    }

    pub fn raw(&self) -> u64 {
        (self.version as u64) << 32 | self.index as u64
    }
}

impl std::fmt::Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntityId({})", self.index)
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
                self.versions.push(0);
                debug_assert!(id < u32::MAX);
                id
            });
        Entity::new(id, self.versions[id as usize])
    }

    pub fn is_alive(&self, id: &Entity) -> bool {
        let version = self.versions[id.index()];
        version < u32::MAX && version == id.version
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
