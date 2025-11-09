use crate::entity::Entity;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Index(usize);

impl Index {
    pub(crate) fn new(data_index: usize) -> Self {
        Self(data_index)
    }

    pub(crate) fn index(&self) -> usize {
        self.0
    }

    pub(crate) fn null() -> Self {
        Self(usize::MAX)
    }

    pub(crate) fn is_null(&self) -> bool {
        self.0 == usize::MAX
    }
}

impl std::fmt::Debug for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            write!(f, "Index::null")
        } else {
            write!(f, "Index({})", self.0)
        }
    }
}

pub struct SparseIndices {
    pub(crate) indices: Vec<Index>,
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
            indices: vec![Index::null(); capacity],
        }
    }

    pub(crate) fn get_index(&self, id: &Entity) -> Option<&Index> {
        self.indices.get(id.index()).filter(|i| !i.is_null())
    }

    pub fn get_data_index(&self, id: &Entity) -> Option<usize> {
        self.indices
            .get(id.index())
            .and_then(|i| (!i.is_null()).then_some(i.index()))
    }

    pub fn set_index(&mut self, id: &Entity, data_index: usize) {
        self.resize_if_needed(id);
        self.indices[id.index()] = Index::new(data_index);
    }

    pub fn set_null(&mut self, id: &Entity) {
        self.indices[id.index()] = Index::null()
    }

    pub fn with<F, T>(&self, id: &Entity, f: F) -> Option<T>
    where
        F: FnOnce(usize) -> T
    {
        self.get_index(id).map(|index| f(index.index()))
    }

    pub fn contains(&self, id: &Entity) -> bool {
        self.get_index(id).is_some()
    }

    fn resize_if_needed(&mut self, id: &Entity) {
        let index = id.index();
        if index >= self.indices.len() {
            self.resize(index);
        }
    }

    pub(crate) fn resize(&mut self, new_len: usize) {
        self.indices.resize(new_len + 1, Index::null());
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.indices.shrink_to_fit();
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn reset(&mut self) {
        self.indices.clear();
        self.indices.shrink_to_fit();
    }

    /// Iterate over the index of the associated entity
    pub fn iter_entity_index(&self) -> impl Iterator<Item = usize> {
        self.indices
            .iter()
            .enumerate()
            .filter_map(|(i, idx)| (!idx.is_null()).then_some(i))
    }

    /// Iterate over the position of the indexed data
    pub fn iter_data_index(&self) -> impl Iterator<Item = usize> {
        self.indices
            .iter()
            .filter_map(|i| (!i.is_null()).then_some(i.index()))
    }
}
