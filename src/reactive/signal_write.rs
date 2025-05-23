use std::cell::RefCell;
use std::rc::Rc;

use super::{AnySignal, Reactive, ReactiveId};
use super::traits::Set;

#[derive(Debug)]
pub struct SignalWrite<T> {
    id: ReactiveId,
    value: Rc<RefCell<T>>,
}

impl<T: 'static> SignalWrite<T> {
    pub fn new(id: ReactiveId, value: Rc<RefCell<T>>) -> Self {
        Self { id, value }
    }
}

impl<T> Clone for SignalWrite<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: Rc::clone(&self.value),
        }
    }
}

impl<T> Reactive for SignalWrite<T> {
    fn id(&self) -> ReactiveId { self.id }
}

impl<T> Set for SignalWrite<T> {
    type Value = T;
    fn set(&self, f: impl FnOnce(&mut Self::Value)) {
        f(&mut self.value.borrow_mut());
        // notify
    }
}

impl<T> PartialEq for SignalWrite<T>
where
    T: PartialEq + Eq + Clone + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }
}

impl<T> Eq for SignalWrite<T> where T: PartialEq + Eq + Clone + 'static {}
