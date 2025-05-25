use std::{cell::RefCell, marker::PhantomData};

use crate::runtime::{ReactiveId, RUNTIME};
use super::traits::{Reactive, Set, Update};

#[derive(Clone, Copy)]
pub struct SignalWrite<T> {
    pub(crate) id: ReactiveId,
    pub(crate) phantom: PhantomData<T>
}

impl<T: 'static> SignalWrite<T> {
    pub fn new(id: ReactiveId) -> Self {
        Self { id, phantom: PhantomData }
    }
}

impl<T: Clone + 'static> Reactive for SignalWrite<T> {
    type Value = T;
    fn id(&self) -> ReactiveId { self.id }
}

impl<T: Clone + 'static> Set for SignalWrite<T> {
    fn set(&self, value: Self::Value) {
        RUNTIME.with(|rt| {
            let mut storage = rt.storage.borrow_mut();
            if let Some(v) = storage.get_mut(&self.id()) {
                let v = v.downcast_mut::<RefCell<T>>().unwrap();
                *v.get_mut() = value;
            }
            drop(storage);
            rt.notify_subscribers(self.id);
        })
    }
}

impl<T: Clone + 'static> Update for SignalWrite<T> {
    fn update(&self, f: impl FnOnce(&mut Self::Value)) {
        RUNTIME.with(|rt| {
            let mut storage = rt.storage.borrow_mut();
            if let Some(v) = storage.get_mut(&self.id()) {
                f(v.downcast_mut::<RefCell<T>>().unwrap().get_mut());
            }
            drop(storage);
            rt.notify_subscribers(self.id);
        })
    }
}
