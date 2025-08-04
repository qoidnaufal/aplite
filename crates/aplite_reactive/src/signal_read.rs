use std::sync::{Arc, RwLock};

use crate::graph::{ReactiveNode, GRAPH};
use crate::stored_value::StoredValue;
use crate::reactive_traits::*;
use crate::signal::Signal;
use crate::signal_write::SignalWrite;

#[derive(Clone, Copy)]
pub struct SignalRead<T> {
    pub(crate) node: ReactiveNode<Arc<RwLock<StoredValue<T>>>>,
}

impl<T: 'static> SignalRead<T> {
    pub(crate) fn new(node: ReactiveNode<Arc<RwLock<StoredValue<T>>>>) -> Self {
        Self { node }
    }

    pub fn as_signal(&self) -> Signal<T> {
        Signal { node: self.node }
    }
}

impl<T: 'static> Track for SignalRead<T> {
    fn track(&self) {
        #[cfg(test)] eprintln!(" └─ [TRACKING]  : {self:?}");
        GRAPH.with(|graph| {
            if let Some(any) = graph.get(&self.node)
            && let Some(stored_value) = any.downcast_ref::<Arc<RwLock<StoredValue<T>>>>()
            {
                stored_value.track();
            }
        })
    }

    fn untrack(&self) {
        #[cfg(test)] eprintln!("[UNTRACKING]: {self:?}");
        GRAPH.with(|graph| {
            if let Some(any) = graph.get(&self.node)
            && let Some(stored_value) = any.downcast_ref::<Arc<RwLock<StoredValue<T>>>>()
            {
                stored_value.untrack();
            }
        })
    }
}

impl<T: 'static> Read for SignalRead<T> {
    type Value = T;

    fn read<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        GRAPH.with(|graph| {
            let any = graph.get(&self.node).unwrap();
            let lock = any.downcast_ref::<Arc<RwLock<StoredValue<Self::Value>>>>().unwrap();
            let stored = lock.read().unwrap();
            f(&stored.value)
        })
    }
}

impl<T: Clone + 'static> Get for SignalRead<T> {
    type Value = T;

    fn get_untracked(&self) -> Self::Value {
        self.read(Clone::clone)
    }
}

impl<T: 'static> With for SignalRead<T> {
    type Value = T;

    fn with_untracked<F, R>(&self, f: F) -> R where F: FnOnce(&Self::Value) -> R {
        self.read(f)
    }
}

impl<T: 'static> Dispose for SignalRead<T> {
    fn dispose(&self) { self.as_signal().dispose() }

    fn is_disposed(&self) -> bool {
        GRAPH.with(|graph| graph.get(&self.node).is_none())
    }
}

impl<T> PartialEq for SignalRead<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node.eq(&other.node)
    }
}

impl<T> PartialEq<Signal<T>> for SignalRead<T> {
    fn eq(&self, other: &Signal<T>) -> bool {
        self.node.eq(&other.node)
    }
}

impl<T> PartialEq<SignalWrite<T>> for SignalRead<T> {
    fn eq(&self, other: &SignalWrite<T>) -> bool {
        self.node.eq(&other.node)
    }
}

impl<T> PartialOrd for SignalRead<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.node.partial_cmp(&other.node)
    }
}

impl<T> PartialOrd<Signal<T>> for SignalRead<T> {
    fn partial_cmp(&self, other: &Signal<T>) -> Option<std::cmp::Ordering> {
        self.node.partial_cmp(&other.node)
    }
}

impl<T> PartialOrd<SignalWrite<T>> for SignalRead<T> {
    fn partial_cmp(&self, other: &SignalWrite<T>) -> Option<std::cmp::Ordering> {
        self.node.partial_cmp(&other.node)
    }
}

impl<T: 'static> std::fmt::Debug for SignalRead<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalRead")
            .field("id", &self.node.id)
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}
