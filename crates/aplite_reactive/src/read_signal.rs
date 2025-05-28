use std::marker::PhantomData;

use crate::runtime::ReactiveId;
use crate::traits::*;

#[derive(Clone, Copy)]
pub struct SignalRead<T> {
    pub(crate) id: ReactiveId,
    pub(crate) phantom: PhantomData<T>,
}

impl<T: 'static> SignalRead<T> {
    pub(crate) fn new(id: ReactiveId) -> Self {
        Self { id, phantom: PhantomData }
    }
}

impl<T: 'static> Reactive for SignalRead<T> {
    fn id(&self) -> ReactiveId { self.id }
}

impl<T: 'static> Track for SignalRead<T> {
    type Value = T;
}

// impl<T: Clone + 'static> Get for SignalRead<T> {
//     fn get(&self) -> Self::Value {
//         RUNTIME.with(|rt| {
//             rt.add_subscriber(self.id());
//             let storage = rt.storage.borrow();
//             let v = storage.get(&self.id()).unwrap();
//             let v = v.downcast_ref::<RefCell<T>>().unwrap();
//             v.borrow().clone()
//         })
//     }
// }

// impl<T: 'static> With for SignalRead<T> {
//     fn with<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
//         RUNTIME.with(|rt| {
//             rt.add_subscriber(self.id());
//             let storage = rt.storage.borrow();
//             let v = storage.get(&self.id()).unwrap();
//             f(&v.downcast_ref::<RefCell<T>>().unwrap().borrow())
//         })
//     }
// }
