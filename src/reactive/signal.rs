use std::sync::{Arc, RwLock};

use super::{AnySignal, SignalId};
use super::traits::{Get, Set};

pub fn signal<T>(value: T) -> Signal<T>
where T: Send + Sync + 'static,
{
    Signal::new(value)
}

#[derive(Debug)]
pub struct Signal<T> {
    id: SignalId,
    value: Arc<RwLock<T>>,
}

impl<T: Send + Sync + 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        let signal = Self {
            id: SignalId::new(),
            value: Arc::new(RwLock::new(value)),
        };
        signal
    }

    pub(crate) fn id(&self) -> SignalId { self.id }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: Arc::clone(&self.value),
        }
    }
}

impl<T: Clone> Get for Signal<T> {
    type Value = T;
    fn get(&self) -> Self::Value {
        self.value.read().unwrap().clone()
    }
}

impl<T> Set for Signal<T> {
    type Value = T;
    fn set(&self, f: impl FnOnce(&mut Self::Value)) {
        f(&mut self.value.write().unwrap());
    }
}

impl<T> PartialEq for Signal<T>
where
    T: PartialEq + Eq + Clone + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.value, &other.value)
    }
}

impl<T> Eq for Signal<T> where T: PartialEq + Eq + Clone + 'static {}

unsafe impl<T> Send for Signal<T> {}
unsafe impl<T> Sync for Signal<T> {}

impl<T: Send + Sync + Clone + 'static> From<AnySignal> for Signal<T> {
    fn from(any: AnySignal) -> Self {
        Self {
            id: any.id(),
            value: Arc::new(RwLock::new(any.cast())),
        }
    }
}

impl<T: Clone + 'static> From<&AnySignal> for Signal<T> {
    fn from(any: &AnySignal) -> Self {
        Self {
            id: any.id(),
            value: Arc::new(RwLock::new(any.cast())),
        }
    }
}
