use crate::entity::EntityId;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct SparseSetIndex(Option<u32>);

impl SparseSetIndex {
    pub(crate) fn new(data_index: usize) -> Self {
        Self(Some(data_index as _))
    }

    pub(crate) fn get(&self) -> Option<usize> {
        self.0.map(|num| num as usize)
    }

    pub(crate) fn null() -> Self {
        Self(None)
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.0.is_some()
    }
}

impl std::fmt::Debug for SparseSetIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.get() {
            Some(num) => write!(f, "Index({num})"),
            None => write!(f, "Index::null"),
        }
    }
}

pub struct SparseIndices {
    pub(crate) indices: Vec<SparseSetIndex>,
}

impl Default for SparseIndices {
    fn default() -> Self {
        Self {
            indices: Vec::new(),
        }
    }
}

impl SparseIndices {
    pub fn reserve(capacity: usize) -> Self {
        Self {
            indices: vec![SparseSetIndex::null(); capacity],
        }
    }

    #[inline(always)]
    pub fn get_index(&self, id: EntityId) -> Option<usize> {
        self.indices
            .get(id.index())
            .and_then(SparseSetIndex::get)
    }

    pub fn set_index(&mut self, id: EntityId, data_index: usize) {
        self.resize_if_needed(id);
        self.indices[id.index()] = SparseSetIndex::new(data_index);
    }

    pub fn set_null(&mut self, id: EntityId) {
        self.indices[id.index()] = SparseSetIndex::null()
    }

    pub fn with<T>(&self, id: EntityId, f: impl FnOnce(usize) -> T) -> Option<T> {
        self.get_index(id).map(|index| f(index))
    }

    pub fn contains(&self, id: EntityId) -> bool {
        self.indices
            .get(id.index())
            .is_some_and(SparseSetIndex::is_valid)
    }

    fn resize_if_needed(&mut self, id: EntityId) {
        let index = id.index();
        if index >= self.indices.len() {
            self.resize(index);
        }
    }

    pub(crate) fn resize(&mut self, new_len: usize) {
        self.indices.resize(new_len + 1, SparseSetIndex::null());
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn reset(&mut self) {
        self.indices.clear();
    }

    /// Iterate over the index of the associated entity
    pub fn iter_entity_index(&self) -> impl Iterator<Item = EntityId> {
        self.indices
            .iter()
            .enumerate()
            .filter_map(|(i, idx)| (idx.is_valid()).then_some(EntityId::new(i as _)))
    }

    /// Iterate over the position of the indexed data
    pub fn iter_data_index(&self) -> impl Iterator<Item = usize> {
        self.indices
            .iter()
            .filter_map(SparseSetIndex::get)
    }
}
