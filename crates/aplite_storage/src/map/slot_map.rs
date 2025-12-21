use super::slot::Slot;

/*
#########################################################
#
# Index
#
#########################################################
*/

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotId {
    index: u32,
    version: u32,
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
        state.write_u32(self.index);
    }
}

impl std::fmt::Debug for SlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Index({})", self.index)
    }
}

/*
#########################################################
#
# IndexMap
#
#########################################################
*/

pub struct SlotMap<T> {
    pub(crate) inner: Vec<Slot<T>>,
    next: u32,
    count: u32,
}

impl<T: Clone> Clone for SlotMap<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            next: self.next,
            count: self.count,
        }
    }
}

impl<T> Default for SlotMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SlotMap<T> {
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            next: 0,
            count: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            next: 0,
            count: 0,
        }
    }

    /// Panics if there are already u32::MAX of stored elements. Use [`try_insert`](IndexMap::try_insert) if you want to handle the error manually.
    #[inline(always)]
    pub fn insert(&mut self, data: T) -> SlotId {
        self.try_insert(data).unwrap()
    }

    #[inline(always)]
    pub fn try_insert(&mut self, data: T) -> Result<SlotId, IndexMapError<T>> {
        if self.count == u32::MAX { return Err(IndexMapError::ReachedMaxCapacity(data)) }

        match self.inner.get_mut(self.next as usize) {
            // after removal
            Some(slot) => slot.try_occupy(data)
                .map(|next| {
                    let id = SlotId::new(self.next, slot.version);
                    self.next = next;
                    self.count += 1;
                    id
                })
                .ok_or(IndexMapError::InvalidSlot),
            None => {
                let entity = SlotId::new(self.next, 0);
                self.inner.push(Slot::with_data(data));
                self.count += 1;
                self.next += 1;

                Ok(entity)
            },
        }
    }

    /// Return None if the index is invalid.
    /// Use [`try_replace`](IndexMap::try_replace()) if you want to handle the error manually
    #[inline(always)]
    pub fn replace(&mut self, index: &SlotId, data: T) -> Option<T> {
        self.try_replace(index, data).ok()
    }

    #[inline(always)]
    pub fn try_replace(&mut self, index: &SlotId, data: T) -> Result<T, IndexMapError<T>> {
        match self.inner.get_mut(index.index()) {
            Some(slot) if index.version == slot.version => {
                slot.get_content_mut()
                    .ok_or(IndexMapError::InvalidSlot)
                    .map(|prev| core::mem::replace(prev, data))
            },
            _ => Err(IndexMapError::InvalidIndex(data)),
        }
    }

    #[inline(always)]
    pub fn get(&self, index: &SlotId) -> Option<&T> {
        self.inner
            .get(index.index())
            .and_then(|slot| {
                if slot.version == index.version {
                    slot.get_content()
                } else {
                    None
                }
            })
    }

    #[inline(always)]
    pub fn get_mut(&mut self, index: &SlotId) -> Option<&mut T> {
        self.inner
            .get_mut(index.index())
            .and_then(|slot| {
                if index.version == slot.version {
                    slot.get_content_mut()
                } else {
                    None
                }
            })
    }

    #[inline(always)]
    pub fn remove(&mut self, index: SlotId) -> Option<T> {
        self.inner
            .get_mut(index.index())
            .and_then(|slot| {
                (slot.version == index.version).then_some({
                    slot.try_replace_with(self.next).inspect(|_| {
                        self.next = index.index;
                        self.count -= 1;
                    })
                })?
            })
    }

    #[inline(always)]
    pub fn contains(&self, index: &SlotId) -> bool {
        self.inner
            .get(index.index())
            .is_some_and(|slot| slot.version == index.version)
    }

    #[inline(always)]
    pub fn len(&self) -> usize { self.count as usize }

    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.count == 0 }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.inner.clear();
        self.inner.push(Slot::new());
        self.next = 0;
        self.count = 0;
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (SlotId, &T)> {
        self.into_iter()
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (SlotId, &mut T)> {
        self.into_iter()
    }
}

impl<T> std::fmt::Debug for SlotMap<T>
where
    T: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.iter())
            .finish()
    }
}

impl<T> std::ops::Index<SlotId> for SlotMap<T> {
    type Output = T;

    fn index(&self, index: SlotId) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

impl<T> std::ops::IndexMut<SlotId> for SlotMap<T> {
    fn index_mut(&mut self, index: SlotId) -> &mut Self::Output {
        self.get_mut(&index).unwrap()
    }
}

pub enum IndexMapError<T> {
    ReachedMaxCapacity(T),
    InvalidIndex(T),
    InvalidSlot,
}

impl<T> std::fmt::Debug for IndexMapError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            IndexMapError::ReachedMaxCapacity(_) => "ReachedMaxCapacity",
            IndexMapError::InvalidIndex(_) => "InvalidIndex",
            IndexMapError::InvalidSlot => "InvalidSlot",
        };

        write!(f, "{out}")
    }
}

impl<T> std::fmt::Display for IndexMapError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl<T> std::error::Error for IndexMapError<T> {}

#[cfg(test)]
mod slot_map_test {
    use super::*;

    #[test]
    fn insert_get() {
        let mut storage = SlotMap::new();
        let mut created_ids = Vec::with_capacity(10);

        for i in 0..10 {
            let id = storage.insert(i);
            created_ids.push(id);
        }

        assert!(created_ids.iter().all(|id| storage.get(id).is_some()));
        assert_eq!(storage.len(), created_ids.len());
    }

    #[test]
    fn remove_index() {
        let mut storage = SlotMap::new();
        let mut created_ids = Vec::with_capacity(10);

        for _ in 0..10 {
            let id = storage.insert(());
            created_ids.push(id);
        }

        created_ids.iter().for_each(|id| storage.remove(*id).unwrap());

        let mut new_ids = Vec::with_capacity(10);
        for _ in 0..10 {
            let new_id = storage.insert(());
            new_ids.push(new_id);
        }

        println!("{storage:#?}");
        assert_ne!(created_ids, new_ids);
    }
}
