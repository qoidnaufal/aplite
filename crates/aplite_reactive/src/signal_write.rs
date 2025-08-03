use std::sync::Arc;

use crate::graph::{ReactiveNode, GRAPH};
use crate::stored_value::StoredValue;
use crate::signal::Signal;
use crate::signal_read::SignalRead;
use crate::reactive_traits::*;

#[derive(Clone, Copy)]
pub struct SignalWrite<T> {
    pub(crate) node: ReactiveNode<Arc<StoredValue<T>>>,
}

impl<T: 'static> SignalWrite<T> {
    pub(crate) fn new(node: ReactiveNode<Arc<StoredValue<T>>>) -> Self {
        Self { node }
    }
}

impl<T: 'static> Reactive for SignalWrite<T> {
    fn dirty(&self) {
        self.notify();
    }

    fn subscribe(&self) {}

    fn unsubscribe(&self) {}
}

impl<T: 'static> Notify for SignalWrite<T> {
    fn notify(&self) {
        #[cfg(test)] eprintln!("\n[NOTIFYING]     : {self:?} is notifying the subscribers");
        GRAPH.with(|graph| {
            if let Some(any) = graph.get(&self.node)
            && let Some(stored_value) = any.downcast_ref::<Arc<StoredValue<T>>>()
            {
                stored_value.dirty();
            }
        })
    }
}

impl<T: 'static> Write for SignalWrite<T> {
    type Value = T;

    fn write_untracked(&self, f: impl FnOnce(&mut Self::Value)) {
        GRAPH.with(|graph| {
            let any = graph.get(&self.node).unwrap();
            let stored_value = any.downcast_ref::<Arc<StoredValue<Self::Value>>>().unwrap();
            let mut v = stored_value.value.write().unwrap();
            f(&mut v);
        });
    }
}

impl<T: 'static> Dispose for SignalWrite<T> {
    fn dispose(&self) {
        GRAPH.with(|graph| {
            if let Some(any) = graph.get(&self.node)
            && let Some(stored_value) = any.downcast_ref::<Arc<StoredValue<T>>>()
            {
                stored_value.untrack();
            }
            graph.storage.borrow_mut().remove(&self.node.id);
        })
    }

    fn is_disposed(&self) -> bool {
        GRAPH.with(|graph| {
            graph.storage.borrow().get(&self.node.id).is_some()
        })
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
