use std::sync::{Arc, RwLock};

use crate::reactive_traits::*;
use crate::subscriber::AnySubscriber;
use crate::source::Source;
// use crate::source::{Source, ToAnySource, AnySource};
use crate::graph::Graph;

pub(crate) struct Value<T> {
    pub(crate) value: T,
    pub(crate) subscribers: Vec<AnySubscriber>,
}

unsafe impl<T> Send for Value<T> {}
unsafe impl<T> Sync for Value<T> {}

impl<T: 'static> Value<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            value,
            subscribers: Default::default(),
        }
    }
}

impl<T: 'static> Notify for RwLock<Value<T>> {
    fn notify(&self) {
        if let Ok(mut this) = self.write() {
            this.subscribers
                .drain(..)
                .for_each(|any_subscriber| any_subscriber.notify());
        }
    }
}

impl<T: 'static> Notify for Arc<RwLock<Value<T>>> {
    fn notify(&self) {
        self.as_ref().notify();
    }
}

impl<T: 'static> Track for RwLock<Value<T>> {
    fn track(&self) {
        Graph::with(|graph| {
            if let Some(current) = graph.current.as_ref() {
                self.add_subscriber(current.clone());
            }
        })
    }

    fn untrack(&self) {
        #[cfg(test)] eprintln!(" └─ [UNTRACKING]: {:?}", self.read().unwrap());
        self.clear_subscribers();
    }
}

impl<T: 'static> Track for Arc<RwLock<Value<T>>> {
    fn track(&self) {
        self.as_ref().track();
    }

    fn untrack(&self) {
        self.as_ref().untrack();
    }
}

impl<T: 'static> Source for RwLock<Value<T>> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        if let Ok(mut this) = self.write()
            && !this.subscribers.contains(&subscriber)
        {
            this.subscribers.push(subscriber);
        }
    }

    fn clear_subscribers(&self) {
        if let Ok(mut this) = self.write() {
            this.subscribers.clear();
        }
    }
}

impl<T: 'static> Source for Arc<RwLock<Value<T>>> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        self.as_ref().add_subscriber(subscriber);
    }

    fn clear_subscribers(&self) {
        self.as_ref().clear_subscribers()
    }
}

// impl<T: 'static> ToAnySource for RwLock<Value<T>> {
//     fn to_any_source(self) -> AnySource {
//         AnySource::new(Arc::new(self))
//     }
// }

// impl<T: 'static> ToAnySource for Arc<RwLock<Value<T>>> {
//     fn to_any_source(self) -> AnySource {
//         AnySource::new(self)
//     }
// }

impl<T> std::fmt::Debug for Value<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredValue")
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}
