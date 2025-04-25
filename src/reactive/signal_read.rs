use std::cell::RefCell;
use std::rc::Rc;

use super::{Reactive, SignalId};
use super::traits::Get;

#[derive(Debug)]
pub struct SignalRead<T> {
    id: SignalId,
    value: Rc<RefCell<T>>,
    // subscriber: Vec<SignalId>,
}

impl<T: 'static> SignalRead<T> {
    pub fn new(id: SignalId, value: Rc<RefCell<T>>) -> Self {
        Self {
            id,
            value,
        }
    }
}

impl<T> Clone for SignalRead<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: Rc::clone(&self.value),
        }
    }
}

impl<T> Reactive for SignalRead<T> {
    fn id(&self) -> SignalId {
        self.id
    }
}

impl<T: Clone> Get for SignalRead<T> {
    type Value = T;
    fn get(&self) -> Self::Value {
        self.value.borrow().clone()
    }
}

impl<T> PartialEq for SignalRead<T>
where
    T: PartialEq + Eq + Clone + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }
}

impl<T> Eq for SignalRead<T> where T: PartialEq + Eq + Clone + 'static {}
