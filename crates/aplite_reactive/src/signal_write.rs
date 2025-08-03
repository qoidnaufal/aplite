use std::sync::{Arc, RwLock};

use crate::graph::{ReactiveNode, GRAPH};
use crate::stored_value::StoredValue;
use crate::signal::Signal;
use crate::signal_read::SignalRead;
use crate::reactive_traits::*;

#[derive(Clone, Copy)]
pub struct SignalWrite<T> {
    pub(crate) node: ReactiveNode<Arc<RwLock<StoredValue<T>>>>,
}

impl<T: 'static> SignalWrite<T> {
    pub(crate) fn new(node: ReactiveNode<Arc<RwLock<StoredValue<T>>>>) -> Self {
        Self { node }
    }

    pub fn as_signal(&self) -> Signal<T> {
        Signal { node: self.node }
    }
}

impl<T: 'static> Notify for SignalWrite<T> {
    fn notify(&self) {
        #[cfg(test)] eprintln!("\n[NOTIFYING]     : {self:?}");
        GRAPH.with(|graph| {
            if let Some(any) = graph.get(&self.node)
            && let Some(stored_value) = any.downcast_ref::<Arc<RwLock<StoredValue<T>>>>()
            {
                stored_value.notify();
            }
        })
    }
}

impl<T: 'static> Write for SignalWrite<T> {
    type Value = T;

    fn write_untracked(&self, f: impl FnOnce(&mut Self::Value)) {
        GRAPH.with(|graph| {
            let any = graph.get(&self.node).unwrap();
            let lock = any.downcast_ref::<Arc<RwLock<StoredValue<Self::Value>>>>().unwrap();
            let mut stored = lock.write().unwrap();
            f(&mut stored.value);
        });
    }
}

impl<T: 'static> Dispose for SignalWrite<T> {
    fn dispose(&self) { self.as_signal().dispose() }

    fn is_disposed(&self) -> bool {
        GRAPH.with(|graph| graph.get(&self.node).is_none())
    }
}

impl<T> PartialEq for SignalWrite<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node.eq(&other.node)
    }
}

impl<T> PartialEq<Signal<T>> for SignalWrite<T> {
    fn eq(&self, other: &Signal<T>) -> bool {
        self.node.eq(&other.node)
    }
}

impl<T> PartialEq<SignalRead<T>> for SignalWrite<T> {
    fn eq(&self, other: &SignalRead<T>) -> bool {
        self.node.eq(&other.node)
    }
}

impl<T> PartialOrd for SignalWrite<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.node.partial_cmp(&other.node)
    }
}

impl<T> PartialOrd<Signal<T>> for SignalWrite<T> {
    fn partial_cmp(&self, other: &Signal<T>) -> Option<std::cmp::Ordering> {
        self.node.partial_cmp(&other.node)
    }
}

impl<T> PartialOrd<SignalRead<T>> for SignalWrite<T> {
    fn partial_cmp(&self, other: &SignalRead<T>) -> Option<std::cmp::Ordering> {
        self.node.partial_cmp(&other.node)
    }
}

impl<T: 'static> std::fmt::Debug for SignalWrite<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalWrite")
            .field("id", &self.node.id)
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}
