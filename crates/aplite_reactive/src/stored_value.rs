use std::any::Any;
use std::sync::{Arc, RwLock};

use crate::subscriber::{Subscriber, WeakSubscriber};

pub(crate) struct StoredValue {
    pub(crate) value: Arc<dyn Any>,
    pub(crate) subscribers: Vec<WeakSubscriber>,
}

impl StoredValue {
    pub(crate) fn new<T: Any + 'static>(value: T) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
            subscribers: Default::default(),
        }
    }

    pub(crate) fn add_subscriber(&mut self, subscriber: WeakSubscriber) {
        if !self.subscribers.contains(&subscriber) {
            self.subscribers.push(subscriber);
        }
    }

    pub(crate) fn notify_subscribers(&self) {
        self.subscribers
            .iter()
            .for_each(|weak_subscriber| {
                weak_subscriber.notify();
            });
    }

    #[inline(always)]
    pub(crate) fn clear_subscribers(&mut self) {
        self.subscribers.clear();
    }

    #[inline(always)]
    pub(crate) fn downcast_ref<T: 'static>(&self) -> Option<&RwLock<T>> {
        self.value.downcast_ref::<RwLock<T>>()
    }
}
