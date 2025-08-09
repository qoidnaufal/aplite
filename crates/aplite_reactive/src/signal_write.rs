use std::sync::{Arc, RwLock};

use crate::graph::{Node, Graph};
use crate::stored_value::Value;
use crate::signal::Signal;
use crate::signal_read::SignalRead;
use crate::reactive_traits::*;

pub struct SignalWrite<T> {
    pub(crate) node: Node<Arc<RwLock<Value<T>>>>,
}

impl<T> Clone for SignalWrite<T> {
    fn clone(&self) -> Self { Self { node: self.node } }
}

impl<T> Copy for SignalWrite<T> {}

impl<T: 'static> SignalWrite<T> {
    pub(crate) fn new(node: Node<Arc<RwLock<Value<T>>>>) -> Self {
        Self { node }
    }

    pub fn as_signal(&self) -> Signal<T> {
        Signal { node: self.node }
    }
}

impl<T: 'static> Notify for SignalWrite<T> {
    fn notify(&self) {
        #[cfg(test)] eprintln!("\n[NOTIFYING]     : {self:?}");
        Graph::with_downcast(&self.node, |node| node.notify())
    }
}

impl<T: 'static> Write for SignalWrite<T> {
    type Value = T;

    fn write(&self, f: impl FnOnce(&mut Self::Value)) {
        Graph::with_downcast(&self.node, |node| {
            let mut stored = node.write().unwrap();
            f(&mut stored.value)
        })
    }
}

impl<T: 'static> Set for SignalWrite<T> {
    type Value = T;

    fn set_untracked(&self, value: Self::Value) {
        self.write(|old| *old = value);
    }
}

impl<T: 'static> Update for SignalWrite<T> {
    type Value = T;

    fn update_untracked(&self, f: impl FnOnce(&mut Self::Value)) {
        self.write(f);
    }
}

impl<T: 'static> Dispose for SignalWrite<T> {
    fn dispose(&self) { self.as_signal().dispose() }

    fn is_disposed(&self) -> bool {
        Graph::is_removed(&self.node)
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
