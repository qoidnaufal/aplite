use std::any::Any;
use std::cell::RefCell;

use crate::read_signal::SignalRead;
use crate::runtime::RUNTIME;
use crate::write_signal::SignalWrite;

type ReactiveValue = Box<dyn Any>;

pub struct Signal {
    value: ReactiveValue,
}

impl Signal {
    pub fn new<T: 'static>(value: T) -> (SignalRead<T>, SignalWrite<T>) {
        RUNTIME.with(|rt| rt.create_rw_signal(value)).split()
    }

    pub fn read_only<T: 'static>(value: T) -> SignalRead<T> {
        RUNTIME.with(|rt| rt.create_rw_signal(value)).read_only()
    }

    pub fn write_only<T: 'static>(value: T) -> SignalWrite<T> {
        RUNTIME.with(|rt| rt.create_rw_signal(value)).write_only()
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
