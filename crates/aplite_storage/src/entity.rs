#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId {
    pub(crate) index: u32,
    pub(crate) version: u32,
}

impl EntityId {
    pub(crate) const fn new(index: u32, version: u32) -> Self {
        Self {
            index,
            version,
        }
    }

    pub const fn root() -> Self {
        Self {
            index: 0,
            version: 0,
        }
    }

    pub fn raw(&self) -> u64 {
        (self.version as u64) << 32 | self.index as u64
    }

    pub fn index(&self) -> usize {
        self.index as usize
    }

    pub const fn version(&self) -> u32 {
        self.version
    }
}

impl std::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntityId({})", self.index)
    }
}

#[derive(Debug)]
pub struct IdManager {
    versions: Vec<u32>,
    recycled: Vec<u32>,
}

impl Default for IdManager {
    /// Create a new manager with no reserved capacity at all.
    /// If you want to preallocate a specific initial capacity, use [`IdManager::with_capacity`]
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl IdManager {
    /// Create a new manager with the specified capacity for the version manager & recycled,
    /// Using [`IdManager::default`] will create one with no preallocated capacity at all
    pub fn with_capacity(capacity: usize) -> Self {
        let mut this = Self {
            recycled: Vec::default(),
            versions: Vec::with_capacity(capacity + 1),
        };

        this.versions.push(0);
        this
    }

    pub fn create(&mut self) -> EntityId {
        let id = self.recycled
            .pop()
            .unwrap_or_else(|| {
                let id = u32::try_from(self.versions.len())
                    .ok()
                    .expect("Created Entity should not exceed u32::MAX");
                self.versions.push(0);
                assert!(id <= u32::MAX);
                id
            });
        EntityId::new(id, self.versions[id as usize])
    }

    pub fn is_alive(&self, id: &EntityId) -> bool {
        self.versions[id.index()] == id.version()
    }

    pub fn destroy(&mut self, id: EntityId) {
        let idx = id.index();
        self.versions[idx] += 1;
        self.recycled.push(idx as u32);
    }

    pub fn reset(&mut self) {
        self.recycled.clear();
        self.versions.clear();
        self.versions.push(0);
    }
}
