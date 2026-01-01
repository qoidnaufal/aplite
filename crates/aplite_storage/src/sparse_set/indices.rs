use std::num::NonZeroU32;

use crate::SparsetKey;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct SparseSetIndex(pub(crate) Option<NonZeroU32>);

impl SparseSetIndex {
    pub(crate) const fn new(data_index: usize) -> Self {
        unsafe {
            Self(Some(NonZeroU32::new_unchecked(data_index as u32 + 1)))
        }
    }

    pub(crate) fn get(&self) -> Option<usize> {
        self.0.map(|num| (num.get() - 1) as usize)
    }

    pub(crate) const fn null() -> Self {
        Self(None)
    }

    pub(crate) const fn is_valid(&self) -> bool {
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

#[derive(Debug, Clone)]
pub struct SparseIndices(pub(crate) Vec<SparseSetIndex>);

impl Default for SparseIndices {
    fn default() -> Self {
        Self::new()
    }
}

impl SparseIndices {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn reserve(capacity: usize) -> Self {
        Self(vec![SparseSetIndex::null(); capacity])
    }

    #[inline(always)]
    pub unsafe fn get_index_unchecked<K: SparsetKey>(&self, key: K) -> usize {
        (self.0[key.index()].0.unwrap().get() - 1) as usize
    }

    #[inline(always)]
    pub fn get_index<K: SparsetKey>(&self, key: K) -> Option<usize> {
        self.0.get(key.index())
            .and_then(SparseSetIndex::get)
    }

    #[inline(always)]
    pub fn set_index<K: SparsetKey>(&mut self, key: K, data_index: usize) {
        let index = key.index();
        self.resize_if_needed(index);
        self.0[index] = SparseSetIndex::new(data_index);
    }

    #[inline(always)]
    pub fn set_null<K: SparsetKey>(&mut self, key: K) {
        self.0[key.index()] = SparseSetIndex::null()
    }

    #[inline(always)]
    pub fn contains<K: SparsetKey>(&self, key: K) -> bool {
        self.0.get(key.index())
            .is_some_and(SparseSetIndex::is_valid)
    }

    #[inline(always)]
    fn resize_if_needed(&mut self, key: usize) {
        if key >= self.0.len() {
            self.resize(key);
        }
    }

    #[inline(always)]
    pub(crate) fn resize(&mut self, new_len: usize) {
        self.0.resize(new_len + 1, SparseSetIndex::null());
    }

    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline(always)]
     pub fn iter_key_id(&self) -> impl Iterator<Item = usize> {
         self.0.iter().enumerate()
             .filter_map(|(i, idx)| idx.is_valid().then_some(i))
     }

    /// Iterate over the position of the indexed data
    #[inline(always)]
    pub fn iter_data_index(&self) -> impl Iterator<Item = usize> {
        self.0.iter()
            .filter_map(SparseSetIndex::get)
    }
}
