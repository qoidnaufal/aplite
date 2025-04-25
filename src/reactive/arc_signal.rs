use std::sync::{Arc, RwLock};

use super::{AnySignal, Get, Reactive, Set, SignalId};

#[derive(Debug)]
pub struct ArcSignal<T> {
    id: SignalId,
    value: Arc<RwLock<T>>,
}

impl<T: Send + Sync + 'static> ArcSignal<T> {
    pub fn new(value: T) -> Self {
        let signal = Self {
            id: SignalId::new(),
            value: Arc::new(RwLock::new(value)),
        };
        signal
    }
}

impl<T> Clone for ArcSignal<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: Arc::clone(&self.value),
        }
    }
}

impl<T> Reactive for ArcSignal<T> {
    fn id(&self) -> SignalId {
        self.id
    }
}

impl<T: Clone> Get for ArcSignal<T> {
    type Value = T;
    fn get(&self) -> Self::Value {
        self.value.read().unwrap().clone()
    }
}

impl<T> Set for ArcSignal<T> {
    type Value = T;
    fn set(&self, f: impl FnOnce(&mut Self::Value)) {
        f(&mut self.value.write().unwrap());
    }
}

impl<T> PartialEq for ArcSignal<T>
where
    T: PartialEq + Eq + Clone + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.value, &other.value)
    }
}

impl<T> Eq for ArcSignal<T> where T: PartialEq + Eq + Clone + 'static {}

unsafe impl<T> Send for ArcSignal<T> {}
unsafe impl<T> Sync for ArcSignal<T> {}
