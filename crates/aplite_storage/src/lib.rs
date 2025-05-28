use std::marker::PhantomData;

#[derive(Debug)]
pub enum SlotContent<V> {
    Occupied(V),
    Vacant(u32),
}

impl<V> SlotContent<V> {
    fn get(&self) -> Option<&V> {
        match self {
            SlotContent::Occupied(v) => Some(v),
            SlotContent::Vacant(_) => None,
        }
    }

    fn get_mut(&mut self) -> Option<&mut V> {
        match self {
            SlotContent::Occupied(v) => Some(v),
            SlotContent::Vacant(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct Slot<V> {
    content: SlotContent<V>,
    next: u32,
}

impl<V> Slot<V> {
    fn vacant(idx: u32) -> Self {
        Self {
            content: SlotContent::Vacant(idx),
            next: idx + 1,
        }
    }

    fn occupied(v: V, current: u32) -> Self {
        Self {
            content: SlotContent::Occupied(v),
            next: current + 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeyHandle<K: Key> {
    idx: u32,
    phantom: PhantomData<K>,
}

impl<K: Key> KeyHandle<K> {
    fn new(idx: u32) -> Self {
        Self {
            idx,
            phantom: PhantomData,
        }
    }

    pub fn idx(&self) -> u32 { self.idx }
}

#[derive(Debug)]
pub struct Map<K: Key, V> {
    inner: Vec<Slot<V>>,
    free_slot: u32,
    count: u32,
    phantom: PhantomData<K>,
}

pub trait Key: Sized {
    fn idx(&self) -> u32;
    fn from_handle(handle: &KeyHandle<Self>) -> Self;
}

impl<K: Key, V> Map<K, V> {
    pub fn new() -> Self {
        Self::new_with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::new_with_capacity(capacity)
    }

    fn new_with_capacity(capacity: usize) -> Self {
        let mut inner = Vec::with_capacity(capacity + 1);
        let slot = Slot::vacant(0);
        inner.push(slot);
        Self {
            inner,
            free_slot: 0,
            count: 0,
            phantom: PhantomData,
        }
    }

    pub fn insert(&mut self, v: V) -> K {
        if self.count + 1 == u32::MAX { panic!("reached max capacity") }

        self.try_insert_with_key(|h| Ok((K::from_handle(h), v))).unwrap()
    }

    pub fn insert_with_key<F>(&mut self, f: F) -> K
    where F: FnOnce(&KeyHandle<K>) -> Result<(K, V), ()>
    { self.try_insert_with_key(f).unwrap() }

    pub fn try_insert_with_key<F>(&mut self, f: F) -> Result<K, ()>
    where
        F: FnOnce(&KeyHandle<K>) -> Result<(K, V), ()>,
    {
        if self.count + 1 == u32::MAX { panic!("reached max capacity") }

        if let Some(slot) = self.inner.get_mut(self.free_slot as usize) {
            // FIXME: this looks stupid?
            let idx = match slot.content {
                SlotContent::Occupied(_) => self.free_slot,
                SlotContent::Vacant(idx) => idx,
            };
            let handle = KeyHandle::new(idx);
            let (k, v) = f(&handle)?;

            slot.content = SlotContent::Occupied(v);
            self.count += 1;

            self.inner.push(Slot::vacant(self.count));
            self.free_slot += 1;

            return Ok(k);
        }

        let last = self.inner.last().unwrap();
        let handle = KeyHandle::new(last.next);
        let (k, v) = f(&handle)?;
        self.inner.push(Slot::occupied(v, last.next));

        Ok(k)
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.inner
            .get(k.idx() as usize)
            .map(|slot| slot.content.get())
            .flatten()
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.inner
            .get_mut(k.idx() as usize)
            .map(|slot| slot.content.get_mut())
            .flatten()
    }
}

#[cfg(test)]
mod slot_test {
    use super::*;

    #[derive(Debug)]
    struct MyKey(u32);

    impl Key for MyKey {
        fn idx(&self) -> u32 {
            self.0
        }

        fn from_handle(handle: &KeyHandle<Self>) -> Self {
            Self(handle.idx())
        }
    }

    #[test]
    fn slot_insert() {
        let mut storage: Map<MyKey, u8> = Map::new();
        let key = storage.insert(69);
        assert!(key.idx() == 0)
    }

    #[test]
    fn slot_get() {
        let mut storage: Map<MyKey, u8> = Map::new();
        let key = storage.insert(69);
        let res = storage.get(&key).unwrap();
        assert!(*res == 70)
    }
}
