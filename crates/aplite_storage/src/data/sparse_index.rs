use crate::entity::EntityId;

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

pub struct SparseIndices {
    pub(crate) ptr: Vec<Index>,
}

impl Default for SparseIndices {
    fn default() -> Self {
        Self {
            ptr: Vec::new(),
        }
    }
}

impl SparseIndices {
    pub fn reserve(capacity: usize) -> Self {
        Self {
            ptr: vec![Index::null(); capacity],
        }
    }

    pub(crate) fn get_index(&self, id: &EntityId) -> Option<&Index> {
        self.ptr.get(id.index()).filter(|i| !i.is_null())
    }

    pub fn get_data_index(&self, id: &EntityId) -> Option<usize> {
        self.ptr
            .get(id.index())
            .and_then(|i| (!i.is_null()).then_some(i.index()))
    }

    pub fn set_index(&mut self, id: &EntityId, data_index: usize) {
        self.resize_if_needed(id);
        self.ptr[id.index()] = Index::new(data_index);
    }

    pub fn set_null(&mut self, id: &EntityId) {
        self.ptr[id.index()] = Index::null()
    }

    pub fn with<F, T>(&self, id: &EntityId, f: F) -> Option<T>
    where
        F: FnOnce(usize) -> T
    {
        self.get_index(id).map(|index| f(index.index()))
    }

    pub fn contains(&self, id: &EntityId) -> bool {
        self.get_index(id).is_some()
    }

    fn resize_if_needed(&mut self, id: &EntityId) {
        let index = id.index();
        if index >= self.ptr.len() {
            self.resize(index);
        }
    }

    pub(crate) fn resize(&mut self, new_len: usize) {
        self.ptr.resize(new_len + 1, Index::null());
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.ptr.shrink_to_fit();
    }

    pub fn len(&self) -> usize {
        self.ptr.len()
    }

    pub fn reset(&mut self) {
        self.ptr.clear();
        self.ptr.shrink_to_fit();
    }

    /// Iterate over the index of the associated entity
    pub fn iter_entity_index(&self) -> impl Iterator<Item = usize> {
        self.ptr
            .iter()
            .enumerate()
            .filter_map(|(i, idx)| (!idx.is_null()).then_some(i))
    }

    /// Iterate over the position of the indexed data
    pub fn iter_data_index(&self) -> impl Iterator<Item = usize> {
        self.ptr
            .iter()
            .filter_map(|i| (!i.is_null()).then_some(i.index()))
    }
}
