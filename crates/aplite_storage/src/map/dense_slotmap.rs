use crate::map::slot_map::SlotId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Slot {
    /// vacant means this is the recycled_id, occupied means this is the the data index
    id: u32,

    /// even: occupied, odd: empty
    version: u32,
}

#[derive(Default)]
/// A hybrid of SparseSet and SlotMap
/// This one is prioritized for iteration-style access. For a frequent access via lookup, normal SlotMap is much better.
pub struct DenseSlotMap<T> {
    /// The place where data are densely stored
    pub(crate) data: Vec<T>,

    /// The sparse indexer, and (together with next) act as next_id generator
    pub(crate) slots: Vec<Slot>,
    next: u32,
}

impl<T> DenseSlotMap<T> {
    pub const fn new() -> Self {
        Self {
            data: Vec::new(),
            slots: Vec::new(),
            next: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            slots: Vec::new(),
            next: 0,
        }
    }

    /// Panics if there are already u32::MAX of stored elements. Use [`try_insert`](IndexMap::try_insert) if you want to handle the error manually.
    pub fn insert(&mut self, data: T) -> SlotId {
        self.try_insert(data).unwrap()
    }

    pub fn try_insert(&mut self, data: T) -> Result<SlotId, Returned<T>> {
        if self.len() as u32 == u32::MAX { return Err(Returned(data)) }

        match self.slots.get_mut(self.next as usize) {
            // after removal
            Some(slot) => {
                debug_assert_ne!(slot.version % 2, 0);

                let next_id = slot.id;
                slot.version += 1;

                let id = SlotId::new(self.next, slot.version);

                slot.id = self.data.len() as u32;
                self.next = next_id;
                self.data.push(data);

                Ok(id)
            }
            None => {
                let id = SlotId::new(self.next, 0);
                self.slots.push(Slot { id: self.data.len() as u32, version: 0 });
                self.data.push(data);
                self.next += 1;
                Ok(id)
            },
        }
    }

    pub fn remove(&mut self, id: SlotId) -> Option<T> {
        let remove_info = self.slots
            .get_mut(id.index())
            .and_then(|slot| {
                if slot.version % 2 != 0 { return None }

                let to_remove_id = slot.id; // this is current data index it points to
                let last_id = self.data.len() - 1;

                slot.id = self.next;
                self.next = id.index;

                let removed = self.data.swap_remove(to_remove_id as usize);
                slot.version += 1;

                Some((removed, to_remove_id, last_id))
            });

        if let Some((removed, to_remove_id, last_id)) = remove_info {
            let swapped_slot = self.slots.get_mut(last_id).unwrap();
            swapped_slot.id = to_remove_id;
            return Some(removed);
        }

        None
    }

    pub fn get(&self, id: SlotId) -> Option<&T> {
        let slot = &self.slots[id.index()];
        if slot.version % 2 != 0 { return None }

        Some(&self.data[slot.id as usize])
    }

    pub fn get_mut(&mut self, id: SlotId) -> Option<&mut T> {
        let slot = &self.slots[id.index()];
        if slot.version % 2 != 0 { return None }

        Some(&mut self.data[slot.id as usize])
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.data.clear();
        self.next = 0;
    }

    pub fn slots(&self) -> usize {
        self.slots.len()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.iter_mut()
    }
}

impl<T: Clone> Clone for DenseSlotMap<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            slots: self.slots.clone(),
            next: self.next,
        }
    }
}

pub struct Returned<T>(T);

impl<T> std::fmt::Debug for Returned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Returned")
            .field("_type:", &std::any::type_name::<T>())
            .finish()
    }
}

impl<T> std::fmt::Display for Returned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl<T> std::error::Error for Returned<T> {}

#[cfg(test)]
mod dense_slotmap {
    use super::*;

    #[test]
    fn removal() {
        let mut s = DenseSlotMap::with_capacity(10);

        for i in 0..10 { s.insert(i); }

        println!("{:?}", s.data);
        println!("{:?}\n", s.slots);

        s.remove(SlotId::new(6, 0));
        s.remove(SlotId::new(2, 0));

        println!("{:?}", s.data);
        println!("{:?}\n", s.slots);

        let g1 = s.get(SlotId::new(2, 0));
        assert!(g1.is_none());
        println!("{:?}", g1);

        let n1 = s.insert(69);
        assert_eq!(n1.version, 2);
        let n2 = s.insert(88);
        assert_eq!(n2.version, 2);
        println!("i: {} . v: {} | i: {} . v: {}\n", n1.index, n1.version, n2.index, n2.version);

        println!("{:?}", s.data);
        println!("{:?}\n", s.slots);

        let g2 = s.get(n2);
        assert!(g2.is_some());
        println!("{:?}", g2);
    }
}
