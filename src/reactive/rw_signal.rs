use std::cell::RefCell;
use std::rc::Rc;

use super::{AnySignal, Reactive, SignalId, SignalRead, SignalWrite};
use super::traits::{Get, Set};

#[derive(Debug)]
pub struct RwSignal<T> {
    id: SignalId,
    value: Rc<RefCell<T>>,
    // subscriber: Vec<SignalId>,
}

impl<T: 'static> RwSignal<T> {
    pub fn new(value: T) -> Self {
        let rw_signal = Self {
            id: SignalId::new(),
            value: Rc::new(RefCell::new(value)),
        };
        rw_signal
    }

    pub fn split(self) -> (SignalRead<T>, SignalWrite<T>) {
        (
            SignalRead::new(self.id, Rc::clone(&self.value)),
            SignalWrite::new(self.id, Rc::clone(&self.value))
        )
    }
}

impl<T> Clone for RwSignal<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: Rc::clone(&self.value),
        }
    }
}

impl<T> Reactive for RwSignal<T> {
    fn id(&self) -> SignalId { self.id }
}

impl<T: Clone> Get for RwSignal<T> {
    type Value = T;
    fn get(&self) -> Self::Value {
        self.value.borrow().clone()
    }
}

impl<T> Set for RwSignal<T> {
    type Value = T;
    fn set(&self, f: impl FnOnce(&mut Self::Value)) {
        f(&mut self.value.borrow_mut());
    }
}

impl<T> PartialEq for RwSignal<T>
where
    T: PartialEq + Eq + Clone + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }
}

impl<T> Eq for RwSignal<T> where T: PartialEq + Eq + Clone + 'static {}
