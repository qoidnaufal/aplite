#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotId {
    pub(crate) index: u32,
    pub(crate) version: u32,
}

impl SlotId {
    pub const fn new(index: u32, version: u32) -> Self {
        Self {
            index,
            version,
        }
    }

    pub const fn version(&self) -> u32 {
        self.version
    }

    pub const fn index(&self) -> usize {
        self.index as _
    }

    pub const fn raw(&self) -> u64 {
        (self.version as u64) << 32 | self.index as u64
    }
}

impl std::hash::Hash for SlotId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.raw());
    }
}

impl std::fmt::Debug for SlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SlotId({})", self.index)
    }
}
