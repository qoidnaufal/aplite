use std::any::Any;
use std::cell::RefCell;

use crate::read_signal::SignalRead;
use crate::runtime::{ReactiveId, RUNTIME};
use crate::write_signal::SignalWrite;

type ReactiveValue = Box<dyn Any>;

pub struct Signal {
    value: ReactiveValue,
}

impl Signal {
    pub fn new<T: 'static>(value: T) -> (SignalRead<T>, SignalWrite<T>) {
        RUNTIME.with(|rt| rt.create_rw_signal(ReactiveId::new(), value)).split()
    }

    pub(crate) fn stored<T: Any + 'static>(value: T) -> Self {
        Self {
            value: Box::new(RefCell::new(value)),
        }
    }

    pub(crate) fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.value.downcast_ref::<T>()
    }

    pub(crate) fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.value.downcast_mut::<T>()
    }
}

// impl<T: 'static> Reactive for Signal<T> {
//     fn id(&self) -> ReactiveId { self.id }
// }

// impl<T: 'static> Track for Signal<T> {
//     type Value = T;
// }

// impl<T: 'static> Notify for Signal<T> {
//     type Value = T;
// }
