#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitset(pub(crate) usize);

impl Bitset {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn update(&mut self, value: usize) {
        self.0 |= 1 << value
    }

    pub fn contains(&self, other: &Bitset) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::hash::Hash for Bitset {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::fmt::Debug for Bitset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BitSet({:b})", self.0)
    }
}
