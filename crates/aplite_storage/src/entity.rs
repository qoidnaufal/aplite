#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(u32);

impl EntityId {
    pub(crate) const VERSION_BITS: u8 = 10;
    pub(crate) const VERSION_MASK: u16 = (1 << Self::VERSION_BITS) - 1;
    pub(crate) const INDEX_BITS: u8 = 22;
    pub(crate) const INDEX_MASK: u32 = (1 << Self::INDEX_BITS) - 1;

    pub(crate) fn new(index: u32, version: u16) -> Self {
        Self((version as u32) << Self::INDEX_BITS | index)
    }

    pub fn root() -> Self {
        Self(0)
    }

    pub fn index(&self) -> usize {
        (self.0 & Self::INDEX_MASK) as usize
    }

    pub(crate) fn version(&self) -> u16 {
        (self.0 >> Self::INDEX_BITS) as u16 & Self::VERSION_MASK
    }
}

impl std::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntityId({})", self.index())
    }
}

#[derive(Debug)]
pub struct IdManager {
    recycled: std::collections::VecDeque<u32>,
    versions: Vec<u16>,
}

impl Default for IdManager {
    /// Create a new manager with no reserved capacity at all.
    /// If you want to preallocate a specific initial capacity, use [`IdManager::with_capacity`]
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl IdManager {
    const MINIMUM_FREE_INDEX: usize = 1 << EntityId::VERSION_BITS;

    /// Create a new manager with the specified capacity for the version manager & recycled,
    /// Using [`EntityManager::default`] will create one with no preallocated capacity at all
    pub fn with_capacity(capacity: usize) -> Self {
        let mut this = Self {
            recycled: std::collections::VecDeque::default(),
            versions: Vec::with_capacity(capacity + 1),
        };

        this.versions.push(0);
        this
    }

    pub fn create(&mut self) -> EntityId {
        let id = if self.recycled.len() >= (Self::MINIMUM_FREE_INDEX) {
            self.recycled.pop_front().unwrap()
        } else {
            let id = self.versions.len() as u32;
            assert!(id <= EntityId::INDEX_MASK);
            self.versions.push(0);
            id
        };

        EntityId::new(id, self.versions[id as usize])
    }

    pub fn is_alive(&self, id: &EntityId) -> bool {
        self.versions[id.index()] == id.version()
    }

    pub fn destroy(&mut self, id: EntityId) {
        let idx = id.index();
        self.versions[idx] += 1;
        self.recycled.push_back(idx as u32);
    }

    pub fn reset(&mut self) {
        self.recycled.clear();
        self.versions.clear();
        self.versions.push(0);
    }
}
