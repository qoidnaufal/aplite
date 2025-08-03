use std::sync::{Arc, RwLock};

use crate::reactive_traits::*;
use crate::subscriber::{AnySubscriber, Subscriber};
use crate::source::{Source, ToAnySource, AnySource};
use crate::graph::GRAPH;

pub(crate) struct StoredValue<T> {
    pub(crate) value: T,
    pub(crate) subscribers: Vec<AnySubscriber>,
}

impl<T: 'static> StoredValue<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            value,
            subscribers: Default::default(),
        }
    }
}

impl<T: 'static> Notify for RwLock<StoredValue<T>> {
    fn notify(&self) {
        if let Ok(this) = self.try_read() {
            this.subscribers
                .iter()
                .for_each(|subscriber| {
                    subscriber.notify();
                });
        }
    }
}

impl<T: 'static> Notify for Arc<RwLock<StoredValue<T>>> {
    fn notify(&self) {
        if let Ok(stored_value) = self.try_read() {
            stored_value
                .subscribers
                .iter()
                .for_each(|subscriber| {
                    subscriber.notify();
                });
        }
    }
}

impl<T: 'static> Track for RwLock<StoredValue<T>> {
    fn track(&self) {}

    fn untrack(&self) {
        #[cfg(test)] eprintln!(" └─ [UNTRACKING]: {:?}", self.read().ok().unwrap());
        self.clear_subscribers();
    }
}

impl<T: 'static> Track for Arc<RwLock<StoredValue<T>>> {
    fn track(&self) {
        GRAPH.with(|graph| {
            if let Some(current) = graph.current.borrow().as_ref() {
                current.add_source(self.clone().to_any_source());
                self.add_subscriber(current.clone());
            }
        })
    }

    fn untrack(&self) {
        self.as_ref().untrack();
    }
}

impl<T: 'static> Source for RwLock<StoredValue<T>> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        if let Ok(mut this) = self.try_write()
        && !this.subscribers.contains(&subscriber)
        {
            this.subscribers.push(subscriber);
        }
    }

    // FIXME: this is slow, find a better way
    fn clear_subscribers(&self) {
        if let Ok(mut this) = self.try_write() {
            GRAPH.with(|graph| {
                if let Some(current) = graph.current
                    .borrow()
                    .as_ref()
                && let Some(idx) = this.subscribers
                    .iter()
                    .position(|s| s == current)
                {
                    this.subscribers.swap_remove(idx);
                }
            })
        }
    }
}

impl<T: 'static> Source for Arc<RwLock<StoredValue<T>>> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        self.as_ref().add_subscriber(subscriber);
    }

    fn clear_subscribers(&self) {
        self.as_ref().clear_subscribers();
    }
}

impl<T: 'static> ToAnySource for RwLock<StoredValue<T>> {
    fn to_any_source(self) -> AnySource {
        AnySource::new(Arc::new(self))
    }
}

impl<T: 'static> ToAnySource for Arc<RwLock<StoredValue<T>>> {
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
