use std::cell::RefCell;
use std::marker::PhantomData;

use crate::read_signal::SignalRead;
use crate::runtime::{ReactiveId, RUNTIME};
use crate::traits::*;
use crate::write_signal::SignalWrite;

#[derive(Clone, Copy)]
pub struct Signal<T> {
    pub(crate) id: ReactiveId,
    pub(crate) phantom: PhantomData<T>,
}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> (SignalRead<T>, SignalWrite<T>) {
        RUNTIME.with(|rt| rt.create_rw_signal(ReactiveId::new(), value)).split()
    }
}

impl<T: 'static> Reactive for Signal<T> {
    type Value = T;
    fn id(&self) -> ReactiveId { self.id }
}

impl<T: Clone + 'static> Get for Signal<T> {
    fn get(&self) -> Self::Value {
        RUNTIME.with(|rt| {
            rt.add_subscriber(self.id());
            let storage = rt.storage.borrow();
            let v = storage.get(&self.id()).unwrap();
            let v = v.downcast_ref::<RefCell<T>>().unwrap();
            v.clone().into_inner()
        })
    }
}

impl<T: 'static> With for Signal<T> {
    fn with<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        RUNTIME.with(|rt| {
            rt.add_subscriber(self.id());
            let storage = rt.storage.borrow();
            let v = storage.get(&self.id()).unwrap();
            f(&v.downcast_ref::<RefCell<T>>().unwrap().borrow())
        })
    }
}

impl<T: 'static> Set for Signal<T> {
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

impl<T: 'static> Update for Signal<T> {
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
