use std::sync::{Arc, RwLock};

use crate::signal_read::SignalRead;
use crate::graph::{Node, Graph};
use crate::signal_write::SignalWrite;
use crate::stored_value::Value;
use crate::reactive_traits::*;
use crate::source::*;
use crate::subscriber::*;

pub struct Signal<T> {
    pub(crate) node: Node<RwLock<Value<T>>>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> Copy for Signal<T> {}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        let node = Graph::insert(Arc::new(RwLock::new(Value::new(value))));

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

impl<T: 'static> Source for Signal<T> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        Graph::with_downcast(&self.node, |node| node.add_subscriber(subscriber))
    }

    fn clear_subscribers(&self) {
        Graph::with_downcast(&self.node, |node| node.clear_subscribers())
    }
}

// impl<T: 'static> ToAnySource for Signal<T> {
//     fn to_any_source(self) -> AnySource {
//         Graph::with_downcast(&self.node, |node| node.clone().to_any_source())
//     }
// }

impl<T: 'static> Track for Signal<T> {
    fn track(&self) {
        #[cfg(test)] eprintln!(" └─ [TRACKING]  : {self:?}");
        Graph::with_downcast(&self.node, |node| node.track());
        // Graph::with(|graph| {
        //     if let Some(current) = graph.current.as_ref() {
        //         current.add_source(self.to_any_source());
        //     }
        // });
    }

    fn untrack(&self) {
        #[cfg(test)] eprintln!("[UNTRACKING]: {self:?}");
        Graph::with_downcast(&self.node, |node| node.untrack())
    }
}

impl<T: 'static> Notify for Signal<T> {
    fn notify(&self) {
        #[cfg(test)] eprintln!("\n[NOTIFYING]     : {self:?}");
        Graph::with_downcast(&self.node, |node| node.notify())
    }
}

impl<T: 'static> Read for Signal<T> {
    type Value = T;

    fn read<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        Graph::with_downcast(&self.node, |node| {
            f(&node.read().unwrap().value)
        })
    }

    fn try_read<R, F: FnOnce(Option<&Self::Value>) -> Option<R>>(&self, f: F) -> Option<R> {
        Graph::try_with_downcast(&self.node, |node| {
            let value = node.and_then(|node| {
                node.try_read().ok()
            });
            f(value.as_ref().map(|v| &v.value))
        })
    }
}

impl<T: Clone + 'static> Get for Signal<T> {
    type Value = T;

    fn get_untracked(&self) -> Self::Value {
        self.read(Clone::clone)
    }

    fn try_get_untracked(&self) -> Option<Self::Value> {
        self.try_read(|n| n.cloned())
    }
}

impl<T: 'static> With for Signal<T> {
    type Value = T;

    fn with_untracked<F, R>(&self, f: F) -> R where F: FnOnce(&Self::Value) -> R {
        self.read(f)
    }

    fn try_with_untracked<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(Option<&Self::Value>) -> Option<R>
    {
        self.try_read(f)
    }
}

impl<T: 'static> Write for Signal<T> {
    type Value = T;

    fn write(&self, f: impl FnOnce(&mut Self::Value)) {
        Graph::with_downcast(&self.node, |node| {
            let mut stored = node.write().unwrap();
            f(&mut stored.value)
        })
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
        if let Some(any) = Graph::remove(&self.node)
        && let Some(stored_value) = any.downcast_ref::<Arc<RwLock<Value<T>>>>()
        {
            stored_value.untrack();
        }
    }

    fn is_disposed(&self) -> bool {
        Graph::is_removed(&self.node)
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
