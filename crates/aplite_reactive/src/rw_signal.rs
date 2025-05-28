use std::marker::PhantomData;

use super::traits::*;
use crate::read_signal::SignalRead;
use crate::runtime::{ReactiveId, RUNTIME};
use crate::write_signal::SignalWrite;

#[derive(Clone, Copy)]
pub struct RwSignal<T> {
    pub(crate) id: ReactiveId,
    pub(crate) phantom: PhantomData<T>
}

impl<T: 'static> RwSignal<T> {
    pub fn new(value: T) -> Self {
        RUNTIME.with(|rt| rt.create_rw_signal(ReactiveId::new(), value))
    }

    pub fn split(self) -> (SignalRead<T>, SignalWrite<T>) {
        (SignalRead::new(self.id), SignalWrite::new(self.id))
    }

    pub fn read_only(&self) -> SignalRead<T> { SignalRead::new(self.id) }

    pub fn write_only(&self) -> SignalWrite<T> { SignalWrite::new(self.id) }
}

impl<T: 'static> Reactive for RwSignal<T> {
    fn id(&self) -> ReactiveId { self.id }
}

impl<T: 'static> Track for RwSignal<T> {
    type Value = T;
}

impl<T: 'static> Notify for RwSignal<T> {
    type Value = T;
}

// impl<T: Clone + 'static> Get for RwSignal<T> {
//     fn get(&self) -> Self::Value {
//         RUNTIME.with(|rt| {
//             rt.add_subscriber(self.id());
//             let storage = rt.storage.borrow();
//             let v = storage.get(&self.id()).unwrap();
//             let v = v.downcast_ref::<RefCell<T>>().unwrap();
//             v.clone().into_inner()
//         })
//     }
// }

// impl<T: 'static> With for RwSignal<T> {
//     fn with<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
//         RUNTIME.with(|rt| {
//             rt.add_subscriber(self.id());
//             let storage = rt.storage.borrow();
//             let v = storage.get(&self.id()).unwrap();
//             f(&v.downcast_ref::<RefCell<T>>().unwrap().borrow())
//         })
//     }
// }

// impl<T: 'static> Set for RwSignal<T> {
//     fn set(&self, value: Self::Value) {
//         RUNTIME.with(|rt| {
//             let mut storage = rt.storage.borrow_mut();
//             if let Some(v) = storage.get_mut(&self.id()) {
//                 let v = v.downcast_mut::<RefCell<T>>().unwrap();
//                 *v.get_mut() = value;
//             }
//             drop(storage);
//             rt.notify_subscribers(self.id);
//         })
//     }
// }

// impl<T: 'static> Update for RwSignal<T> {
//     fn update(&self, f: impl FnOnce(&mut Self::Value)) {
//         RUNTIME.with(|rt| {
//             let mut storage = rt.storage.borrow_mut();
//             if let Some(v) = storage.get_mut(&self.id()) {
//                 f(v.downcast_mut::<RefCell<T>>().unwrap().get_mut());
//             }
//             drop(storage);
//             rt.notify_subscribers(self.id);
//         })
//     }
// }
