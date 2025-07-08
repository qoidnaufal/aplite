use std::cell::RefCell;

use crate::graph::{ReactiveId, GRAPH};

/*
#########################################################
#                                                       #
#                         CORE                          #
#                                                       #
#########################################################
*/

#[doc(hidden)]
pub trait Reactive {
    fn id(&self) -> &ReactiveId;
}

#[doc(hidden)]
pub trait Track: Reactive {
    fn track(&self) {
        GRAPH.with(|graph| graph.track(self.id()))
    }
}

#[doc(hidden)]
pub trait Notify: Reactive {
    fn notify(&self) {
        GRAPH.with(|graph| graph.notify_subscribers(self.id()))
    }
}

/*
#########################################################
#                                                       #
#                     READ & WRITE                      #
#                                                       #
#########################################################
*/

pub trait Read: Reactive {
    type Value: 'static;

    /// read value without tracking the signal
    fn read_untracked<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        GRAPH.with(|graph| {
            let storage = graph.signals.borrow();
            let signal = storage.get(self.id()).unwrap();
            let v = signal.value_ref::<RefCell<Self::Value>>()
                .unwrap();
            f(&v.borrow())
        })
    }
}

pub trait Write: Reactive {
    type Value: 'static;

    /// updating the value without notifying it's subscribers
    fn write_untracked(&self, f: impl FnOnce(&mut Self::Value)) {
        GRAPH.with(|graph| {
            let storage = graph.signals.borrow();
            if let Some(signal) = storage.get(self.id()) {
                let v = signal
                    .value_ref::<RefCell<Self::Value>>()
                    .unwrap();
                f(&mut v.borrow_mut());
            }
        });
    }
}

/*
#########################################################
#                                                       #
#                     TRACK + READ                      #
#                                                       #
#########################################################
*/

pub trait Get
where
    Self: Track + Read,
    <Self as Read>::Value: Clone,
{
    /// track the signal & clone the value
    fn get(&self) -> <Self as Read>::Value {
        self.track();
        self.get_untracked()
    }

    fn get_untracked(&self) -> <Self as Read>::Value {
        self.read_untracked(|val| val.clone())
    }
}

impl<T> Get for T where T: Track + Read, T::Value: Clone, {}

pub trait With: Track + Read {
    /// track the signal & accessing the value without cloning it
    fn with<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        self.track();
        self.read_untracked(f)
    }
}

impl<T> With for T where T: Track + Read {}

/*
#########################################################
#                                                       #
#                     NOTIFY + WRITE                    #
#                                                       #
#########################################################
*/

pub trait Set: Notify + Write {
    /// update the value directly and notify the subscribers
    fn set(&self, value: <Self as Write>::Value) {
        self.set_untracked(value);
        self.notify();
    }

    fn set_untracked(&self, value: <Self as Write>::Value) {
        self.write_untracked(|val| *val = value);
    }
}

impl<T: Notify + Write> Set for T {}

pub trait Update: Notify + Write {
    /// update the value via a closure and notify the subscribers
    fn update(&self, f: impl FnOnce(&mut <Self as Write>::Value)) {
        self.write_untracked(f);
        self.notify();
    }

    fn update_untracked(&self, f: impl FnOnce(&mut <Self as Write>::Value)) {
        self.write_untracked(f);
    }
}

impl<T: Notify + Write> Update for T {}

/*
#########################################################
#                                                       #
#                       DISPOSE                         #
#                                                       #
#########################################################
*/

pub trait Dispose: Reactive {
    /// untrack this signal and then remove it from the reactive system
    /// be careful accessing the value of disposed signal will cause [`panic!()`](core::panic)
    fn dispose(&self) {
        GRAPH.with(|graph| {
            graph.untrack(self.id());
            graph.signals.borrow_mut().remove(self.id());
        })
    }

    /// check if a signal has been disposed or not
    fn is_disposed(&self) -> bool {
        GRAPH.with(|graph| {
            graph.signals.borrow().get(self.id()).is_some()
        })
    }
}

impl<T: Reactive> Dispose for T {}
