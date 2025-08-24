use crate::graph::Graph;
use crate::reactive_traits::*;
use crate::signal::{Signal, SignalNode};
use crate::signal_write::SignalWrite;
use crate::source::*;
use crate::subscriber::*;

pub struct SignalRead<T> {
    pub(crate) node: SignalNode<T>,
}

impl<T> Clone for SignalRead<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> Copy for SignalRead<T> {}

impl<T: 'static> SignalRead<T> {
    pub(crate) fn new(node: SignalNode<T>) -> Self {
        Self { node }
    }

    pub fn as_signal(&self) -> Signal<T> {
        Signal { node: self.node }
    }
}

impl<T: 'static> Source for SignalRead<T> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        Graph::with_downcast(&self.node, |node| node.add_subscriber(subscriber))
    }

    fn clear_subscribers(&self) {
        Graph::with_downcast(&self.node, |node| node.clear_subscribers())
    }
}

impl<T: 'static> ToAnySource for SignalRead<T> {
    fn to_any_source(&self) -> AnySource {
        Graph::with_downcast(&self.node, |node| node.to_any_source())
    }
}

impl<T: 'static> Notify for SignalRead<T> {
    fn notify(&self) {}
}

impl<T: 'static> Track for SignalRead<T> {
    fn track(&self) {
        self.as_signal().track();
    }

    fn untrack(&self) {
        Graph::with_downcast(&self.node, |node| node.untrack())
    }
}

impl<T: 'static> Read for SignalRead<T> {
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

impl<T: Clone + 'static> Get for SignalRead<T> {
    type Value = T;

    fn get_untracked(&self) -> Self::Value {
        self.read(Clone::clone)
    }

    fn try_get_untracked(&self) -> Option<Self::Value> {
        self.try_read(|v| v.cloned())
    }
}

impl<T: 'static> With for SignalRead<T> {
    type Value = T;

    fn with_untracked<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Value) -> R
    {
        self.read(f)
    }

    fn try_with_untracked<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(Option<&Self::Value>) -> Option<R>
    {
        self.try_read(f)
    }
}

impl<T: 'static> Dispose for SignalRead<T> {
    fn dispose(&self) { self.as_signal().dispose() }

    fn is_disposed(&self) -> bool {
        Graph::is_removed(&self.node)
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
