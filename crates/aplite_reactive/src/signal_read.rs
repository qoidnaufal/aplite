use std::sync::Arc;

use crate::graph::{ReactiveNode, GRAPH};
use crate::stored_value::StoredValue;
use crate::reactive_traits::*;
use crate::signal::Signal;
use crate::signal_write::SignalWrite;

#[derive(Clone, Copy)]
pub struct SignalRead<T> {
    pub(crate) node: ReactiveNode<Arc<StoredValue<T>>>,
}

impl<T: 'static> SignalRead<T> {
    pub(crate) fn new(node: ReactiveNode<Arc<StoredValue<T>>>) -> Self {
        Self { node }
    }
}

impl<T: 'static> Reactive for SignalRead<T> {
    fn dirty(&self) {}

    fn subscribe(&self) {
        self.track();
    }

    fn unsubscribe(&self) {
        self.untrack();
    }
}

impl<T: 'static> Track for SignalRead<T> {
    fn track(&self) {
        #[cfg(test)] eprintln!(" └─ [TRACKING]  : {self:?}");
        GRAPH.with(|graph| {
            if let Some(any) = graph.get(&self.node)
            && let Some(stored_value) = any.downcast_ref::<Arc<StoredValue<T>>>()
            {
                stored_value.subscribe();
            }
        })
    }

    // FIXME: should happen from here instead of stored value?
    fn untrack(&self) {
        #[cfg(test)] eprintln!("[UNTRACKING]: {self:?}");
        GRAPH.with(|graph| {
            if let Some(any) = graph.get(&self.node)
            && let Some(stored_value) = any.downcast_ref::<Arc<StoredValue<T>>>()
            {
                stored_value.unsubscribe();
            }
        })
    }
}

impl<T: 'static> Read for SignalRead<T> {
    type Value = T;

    fn read_untracked<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        GRAPH.with(|graph| {
            let any = graph.get(&self.node).unwrap();
            let stored_value = any.downcast_ref::<Arc<StoredValue<Self::Value>>>().unwrap();
            let v = stored_value.value.read().unwrap();
            f(&v)
        })
    }
}

impl<T: 'static> Dispose for SignalRead<T> {
    fn dispose(&self) {
        self.untrack();
        GRAPH.with(|graph| {
            graph.storage.borrow_mut().remove(&self.node.id);
        })
    }

    fn is_disposed(&self) -> bool {
        GRAPH.with(|graph| {
            graph.storage.borrow().get(&self.node.id).is_some()
        })
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
