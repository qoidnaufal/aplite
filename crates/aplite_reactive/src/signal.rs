use std::sync::{Arc, RwLock};

use crate::signal_read::SignalRead;
use crate::graph::{ReactiveNode, GRAPH};
use crate::signal_write::SignalWrite;
use crate::stored_value::StoredValue;
use crate::reactive_traits::*;

#[derive(Clone, Copy)]
pub struct Signal<T> {
    pub(crate) node: ReactiveNode<Arc<RwLock<StoredValue<T>>>>,
}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        let node = GRAPH.with(|graph| {
            let stored_value = Arc::new(RwLock::new(StoredValue::new(value)));
            graph.insert(stored_value)
        });

        Self {
            node,
        }
    }

    pub fn split(value: T) -> (SignalRead<T>, SignalWrite<T>) {
        Self::new(value).into_split()
    }

    pub fn into_split(self) -> (SignalRead<T>, SignalWrite<T>) {
        (
            SignalRead::new(self.node),
            SignalWrite::new(self.node)
        )
    }

    pub fn read_only(&self) -> SignalRead<T> {
        SignalRead::new(self.node)
    }

    pub fn write_only(&self) -> SignalWrite<T> {
        SignalWrite::new(self.node)
    }
}

impl<T: 'static> Track for Signal<T> {
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

impl<T: 'static> Notify for Signal<T> {
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

impl<T: 'static> Read for Signal<T> {
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

impl<T: Clone + 'static> Get for Signal<T> {
    type Value = T;

    fn get_untracked(&self) -> Self::Value {
        self.read(Clone::clone)
    }
}

impl<T: 'static> With for Signal<T> {
    type Value = T;

    fn with_untracked<F, R>(&self, f: F) -> R where F: FnOnce(&Self::Value) -> R {
        self.read(f)
    }
}

impl<T: 'static> Write for Signal<T> {
    type Value = T;

    fn write(&self, f: impl FnOnce(&mut Self::Value)) {
        GRAPH.with(|graph| {
            let any = graph.get(&self.node).unwrap();
            let lock = any.downcast_ref::<Arc<RwLock<StoredValue<Self::Value>>>>().unwrap();
            let mut stored = lock.write().unwrap();
            f(&mut stored.value);
        });
    }
}

impl<T: 'static> Set for Signal<T> {
    type Value = T;

    fn set_untracked(&self, value: Self::Value) {
        self.write(|old| *old = value);
    }
}

impl<T: 'static> Update for Signal<T> {
    type Value = T;

    fn update_untracked(&self, f: impl FnOnce(&mut Self::Value)) {
        self.write(f);
    }
}

impl<T: 'static> Dispose for Signal<T> {
    fn dispose(&self) {
        GRAPH.with(|graph| {
            if let Some(any) = graph.remove(&self.node)
            && let Some(stored_value) = any.downcast_ref::<Arc<RwLock<StoredValue<T>>>>()
            {
                stored_value.untrack();
            }
        })
    }

    fn is_disposed(&self) -> bool {
        GRAPH.with(|graph| graph.get(&self.node).is_none())
    }
}

impl<T> PartialEq for Signal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node.eq(&other.node)
    }
}

impl<T> PartialEq<SignalRead<T>> for Signal<T> {
    fn eq(&self, other: &SignalRead<T>) -> bool {
        self.node.eq(&other.node)
    }
}

impl<T> PartialEq<SignalWrite<T>> for Signal<T> {
    fn eq(&self, other: &SignalWrite<T>) -> bool {
        self.node.eq(&other.node)
    }
}

impl<T> PartialOrd for Signal<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.node.partial_cmp(&other.node)
    }
}

impl<T> PartialOrd<SignalRead<T>> for Signal<T> {
    fn partial_cmp(&self, other: &SignalRead<T>) -> Option<std::cmp::Ordering> {
        self.node.partial_cmp(&other.node)
    }
}

impl<T> PartialOrd<SignalWrite<T>> for Signal<T> {
    fn partial_cmp(&self, other: &SignalWrite<T>) -> Option<std::cmp::Ordering> {
        self.node.partial_cmp(&other.node)
    }
}

impl<T: 'static> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal")
            .field("id", &self.node.id)
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}
