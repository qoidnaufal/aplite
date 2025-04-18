use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use super::{AnySignal, Reactive, SignalId};
use super::traits::{Get, Set};

pub fn signal<T>(value: T) -> Signal<T>
where T: 'static,
{
    Signal::new(value)
}

pub fn arc_signal<T>(value: T) -> ArcSignal<T>
where T: Send + Sync + 'static
{
    ArcSignal::new(value)
}

#[derive(Debug)]
pub struct Signal<T> {
    id: SignalId,
    value: Rc<RefCell<T>>,
    // subscriber: Vec<SignalId>,
}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        let signal = Self {
            id: SignalId::new(),
            value: Rc::new(RefCell::new(value)),
        };
        signal
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: Rc::clone(&self.value),
        }
    }
}

impl<T> Reactive for Signal<T> {
    fn id(&self) -> SignalId {
        self.id
    }
}

impl<T: Clone> Get for Signal<T> {
    type Value = T;
    fn get(&self) -> Self::Value {
        self.value.borrow().clone()
    }
}

impl<T> Set for Signal<T> {
    type Value = T;
    fn set(&self, f: impl FnOnce(&mut Self::Value)) {
        f(&mut self.value.borrow_mut());
    }
}

impl<T> PartialEq for Signal<T>
where
    T: PartialEq + Eq + Clone + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }
}

impl<T> Eq for Signal<T> where T: PartialEq + Eq + Clone + 'static {}

impl<T: Clone + 'static> From<AnySignal> for Signal<T> {
    fn from(any: AnySignal) -> Self {
        Self {
            id: any.id(),
            value: Rc::new(RefCell::new(any.cast())),
        }
    }
}

impl<T: Clone + 'static> From<&AnySignal> for Signal<T> {
    fn from(any: &AnySignal) -> Self {
        Self {
            id: any.id(),
            value: Rc::new(RefCell::new(any.cast())),
        }
    }
}

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

impl<T: Clone + 'static> From<AnySignal> for ArcSignal<T> {
    fn from(any: AnySignal) -> Self {
        Self {
            id: any.id(),
            value: Arc::new(RwLock::new(any.cast())),
        }
    }
}

impl<T: Clone + 'static> From<&AnySignal> for ArcSignal<T> {
    fn from(any: &AnySignal) -> Self {
        Self {
            id: any.id(),
            value: Arc::new(RwLock::new(any.cast())),
        }
    }
}
