use std::sync::{Arc, RwLock};

use crate::reactive_traits::*;
use crate::subscriber::{AnySubscriber, Subscriber};
use crate::source::{Source, ToAnySource, AnySource};
use crate::graph::GRAPH;

pub(crate) struct StoredValue<T> {
    pub(crate) value: RwLock<T>,
    pub(crate) subscribers: RwLock<Vec<AnySubscriber>>,
}

impl<T: 'static> StoredValue<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            value: RwLock::new(value),
            subscribers: RwLock::new(Default::default()),
        }
    }

    pub(crate) fn notify_subscribers(&self) {
        if let Ok(subscribers) = self.subscribers.try_read() {
            subscribers
                .iter()
                .for_each(|subscriber| {
                    subscriber.notify();
                });
        }
    }
}

impl<T: 'static> Reactive for StoredValue<T> {
    fn dirty(&self) {
        self.notify();
    }

    fn subscribe(&self) {
        self.track();
    }

    fn unsubscribe(&self) {
        self.untrack();
    }
}

impl<T: 'static> Reactive for Arc<StoredValue<T>> {
    fn dirty(&self) {
        self.notify();
    }

    fn subscribe(&self) {
        self.track();
    }

    fn unsubscribe(&self) {
        self.untrack();
    }
}

impl<T: 'static> Notify for StoredValue<T> {
    fn notify(&self) {
        self.notify_subscribers();
    }
}

impl<T: 'static> Notify for Arc<StoredValue<T>> {
    fn notify(&self) {
        self.as_ref().notify_subscribers();
    }
}

impl<T: 'static> Track for StoredValue<T> {
    fn track(&self) {
        GRAPH.with(|graph| {
            if let Some(current) = graph.current.borrow().as_ref() {
                if let Ok(mut subscribers) = self.subscribers.try_write()
                && !subscribers.contains(&current)
                {
                    subscribers.push(current.clone());
                }
            }
        })
    }

    fn untrack(&self) {
        #[cfg(test)] eprintln!(" └─ [UNTRACKING]: {self:?}");
        self.clear_subscribers();
    }
}

impl<T: 'static> Track for Arc<StoredValue<T>> {
    fn track(&self) {
        GRAPH.with(|graph| {
            if let Some(current) = graph.current.borrow().as_ref() {
                current.add_source(self.clone().to_any_source());
                self.add_subscriber(current.clone());
            }
        })
    }

    fn untrack(&self) {
        self.as_ref().clear_subscribers();
    }
}

impl<T: 'static> Source for StoredValue<T> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        if let Ok(mut subscribers) = self.subscribers.try_write()
        && !subscribers.contains(&subscriber)
        {
            subscribers.push(subscriber);
        }
    }

    fn clear_subscribers(&self) {
        if let Ok(mut subscribers) = self.subscribers.try_write() {
            subscribers.clear();
        }
    }
}

impl<T: 'static> Source for Arc<StoredValue<T>> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        self.as_ref().add_subscriber(subscriber);
    }

    fn clear_subscribers(&self) {
        self.as_ref().clear_subscribers();
    }
}

impl<T: 'static> ToAnySource for Arc<StoredValue<T>> {
    fn to_any_source(self) -> AnySource {
        AnySource::new(self)
    }
}

impl<T> std::fmt::Debug for StoredValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredValue")
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}
