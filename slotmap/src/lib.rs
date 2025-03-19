#[derive(Clone)]
pub struct Key<K> {
    key: K,
    idx: usize,
}

#[derive(Clone)]
pub struct Slot<K, V> {
    key: Key<K>,
    val: Option<V>,
}

#[derive(Clone)]
pub struct SlotMap<K, V> {
    len: usize,
    data: Vec<Slot<K, V>>,
}

impl<K, V> Default for SlotMap<K, V> {
    fn default() -> Self {
        Self { len: 0, data: vec![] }
    }
}

impl<K, V> SlotMap<K, V>
where
    K: Clone + PartialEq + Eq,
    V: Clone
{
    pub fn new() -> Self { Self::default() }

    pub fn get(&self, k: K) -> Option<&V> {
        if let Some(slot) = self.data.iter().find(|slot| slot.key.key == k) {
            slot.val.as_ref()
        } else { None }
    }

    pub fn get_by_key(&self, k: &Key<K>) -> Option<&V> {
        self.data[k.idx].val.as_ref()
    }

    pub fn insert(&mut self, k: K, v: V) -> Key<K> {
        if let Some(slot) = self.data.iter_mut().find(|slot| slot.key.key == k) {
            slot.val.replace(v);
            slot.key.clone()
        } else {
            let slot_key = Key {
                key: k,
                idx: self.len,
            };
            let slot = Slot {
                key: slot_key.clone(),
                val: Some(v),
            };
            self.data.push(slot);
            self.len += 1;
            slot_key
        }
    }

    pub fn remove(&mut self, k: &Key<K>) -> Option<V> {
        self.data[k.idx].val.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut sm = SlotMap::new();
        let k = "key";
        let v = "val";
        let insert = sm.insert(k, v);
        assert_eq!(k, insert.key);

        sm.remove(&insert);
        assert_eq!(sm.len, 1);
        assert_eq!(sm.data[0].val, None);
    }
}
