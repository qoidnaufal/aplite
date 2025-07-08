use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use crate::read_signal::ReadSignal;
use crate::graph::GRAPH;
use crate::write_signal::WriteSignal;
use crate::Effect;

type ReactiveValue = Rc<dyn Any>;

pub struct Signal {
    value: ReactiveValue,
    subscribers: RefCell<Vec<Effect>>,
}

impl Signal {
    pub fn new<T: 'static>(value: T) -> (ReadSignal<T>, WriteSignal<T>) {
        GRAPH.with(|graph| graph.create_rw_signal(value)).split()
    }

    pub fn read_only<T: 'static>(value: T) -> ReadSignal<T> {
        GRAPH.with(|graph| graph.create_rw_signal(value)).read_only()
    }

    pub fn write_only<T: 'static>(value: T) -> WriteSignal<T> {
        GRAPH.with(|graph| graph.create_rw_signal(value)).write_only()
    }

    pub(crate) fn store_value<T: Any + 'static>(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
            subscribers: Default::default(),
        }
    }

    pub(crate) fn add_subscriber(&self, subscriber: Effect) {
        let mut subscribers = self.subscribers.borrow_mut();
        if !subscribers.contains(&subscriber) {
            subscribers.push(subscriber);
        }
    }

    pub(crate) fn get_subscribers(&self) -> Vec<Effect>  {
        self.subscribers.borrow().clone()
    }

    pub(crate) fn clear_subscribers(&self) {
        self.subscribers.borrow_mut().clear();
    }

    pub(crate) fn value_ref<T: 'static>(&self) -> Option<&T> {
        self.value.downcast_ref::<T>()
    }
}
