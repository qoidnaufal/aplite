use crate::graph::{ReactiveId, GRAPH};

/*
#########################################################
#                                                       #
#                         CORE                          #
#                                                       #
#########################################################
*/

pub trait Reactive {
    fn id(&self) -> &ReactiveId;
}

pub trait Track: Reactive {
    fn track(&self) {
        GRAPH.with(|graph| graph.track(self.id()))
    }

    fn untrack(&self) {
        GRAPH.with(|graph| {
            let mut storage = graph.storage.borrow_mut();
            if let Some(stored_value) = storage.get_mut(&self.id()) {
                stored_value.clear_subscribers();
            }
        })
    }
}

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

pub trait Read: Reactive + Track {
    type Value: 'static;

    /// read and apply function to the value, and track the underying signal
    fn read<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        self.track();
        self.read_untracked(f)
    }

    /// read value without tracking the signal, and apply a function to the value
    fn read_untracked<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        GRAPH.with(|graph| {
            let storage = graph.storage.borrow();
            let value = storage.get(self.id()).unwrap();
            let v = value.downcast_ref::<Self::Value>()
                .unwrap();
            f(&v.read().unwrap())
        })
    }
}

pub trait Write: Reactive {
    type Value: 'static;

    /// updating the value without notifying it's subscribers
    fn write_untracked(&self, f: impl FnOnce(&mut Self::Value)) {
        GRAPH.with(|graph| {
            let storage = graph.storage.borrow();
            if let Some(value) = storage.get(self.id()) {
                let v = value
                    .downcast_ref::<Self::Value>()
                    .unwrap();
                f(&mut v.write().unwrap());
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
    Self: Read,
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

pub trait Dispose: Reactive + Track {
    /// untrack this signal and then remove it from the reactive system
    /// be careful accessing the value of disposed signal will cause [`panic!()`](core::panic)
    fn dispose(&self) {
        self.untrack();
        GRAPH.with(|graph| {
            // graph.untrack(self.id());
            graph.storage.borrow_mut().remove(self.id());
        })
    }

    /// check if a signal has been disposed or not
    fn is_disposed(&self) -> bool {
        GRAPH.with(|graph| {
            graph.storage.borrow().get(self.id()).is_some()
        })
    }
}

impl<T: Reactive + Track> Dispose for T {}
