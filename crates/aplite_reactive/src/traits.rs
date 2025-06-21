use std::cell::RefCell;

use crate::runtime::{ReactiveId, RUNTIME};

// -------------------------------------------------------- //
// -------------------------------------------------------- //
//                                                          //
//                           CORE                           //
//                                                          //
// -------------------------------------------------------- //
// -------------------------------------------------------- //

pub trait Reactive {
    fn id(&self) -> ReactiveId;
}

pub trait Track: Reactive {
    type Value: 'static;
    fn track(&self) {
        RUNTIME.with(|rt| rt.add_subscriber(self.id()))
    }
}

pub trait Notify: Reactive {
    type Value: 'static;
    fn notify(&self) {
        RUNTIME.with(|rt| rt.notify_subscribers(self.id()))
    }
}

// -------------------------------------------------------- //
// -------------------------------------------------------- //
//                                                          //
//                          TRACK                           //
//                                                          //
// -------------------------------------------------------- //
// -------------------------------------------------------- //

pub trait Get: Track {
    type Value: Clone;
    fn get(&self) -> <Self as Get>::Value;
}

impl<T> Get for T
where
    T: Track,
    T::Value: Clone,
{
    type Value = T::Value;
    fn get(&self) -> <Self as Get>::Value {
        self.track();
        RUNTIME.with(|rt| {
            let storage = rt.signals.borrow();
            let signal = storage.get(&self.id()).unwrap();
            let v = signal
                .downcast_ref::<RefCell<<Self as Get>::Value>>()
                .unwrap();
            v.borrow().clone()
        })
    }
}

pub trait With: Track {
    fn with<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R;
}

impl<T: Track> With for T {
    fn with<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        self.track();
        RUNTIME.with(|rt| {
            let storage = rt.signals.borrow();
            let signal = storage.get(&self.id()).unwrap();
            let v = signal
                .downcast_ref::<RefCell<Self::Value>>()
                .unwrap()
                .borrow();
            f(&v)
        })
    }
}

// -------------------------------------------------------- //
// -------------------------------------------------------- //
//                                                          //
//                         NOTIFY                           //
//                                                          //
// -------------------------------------------------------- //
// -------------------------------------------------------- //

pub trait Set: Notify {
    fn set(&self, value: Self::Value);
}

impl<T: Notify> Set for T {
    fn set(&self, value: Self::Value) {
        RUNTIME.with(|rt| {
            let mut storage = rt.signals.borrow_mut();
            if let Some(signal) = storage.get_mut(&self.id()) {
                let v = signal
                    .downcast_mut::<RefCell<Self::Value>>()
                    .unwrap();
                *v.get_mut() = value;
            }
        });
        self.notify();
    }
}

pub trait Update: Notify {
    fn update(&self, f: impl FnOnce(&mut Self::Value));
}

impl<T: Notify> Update for T {
    fn update(&self, f: impl FnOnce(&mut Self::Value)) {
        RUNTIME.with(|rt| {
            let mut storage = rt.signals.borrow_mut();
            if let Some(signal) = storage.get_mut(&self.id()) {
                let v = signal
                    .downcast_mut::<RefCell<Self::Value>>()
                    .unwrap()
                    .get_mut();
                f(v);
            }
        });
        self.notify();
    }
}

// -------------------------------------------------------- //
// -------------------------------------------------------- //
//                                                          //
//                       SUBSCRIBER                         //
//                                                          //
// -------------------------------------------------------- //
// -------------------------------------------------------- //


pub(crate) trait Subscriber {
    fn run(&mut self);
}

// -------------------------------------------------------- //
// -------------------------------------------------------- //
//                                                          //
//                         Observer                         //
//                                                          //
// -------------------------------------------------------- //
// -------------------------------------------------------- //

// trait Observer: Reactive {}
