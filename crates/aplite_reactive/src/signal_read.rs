use crate::graph::ReactiveStorage;
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

    #[inline(always)]
    pub fn as_signal(&self) -> Signal<T> {
        Signal { node: self.node }
    }
}

impl<T: 'static> Reactive for SignalRead<T> {
    fn mark_dirty(&self) {
        self.as_signal().mark_dirty();
    }

    fn try_update(&self) -> bool {
        false
    }
}

impl<T: 'static> Source for SignalRead<T> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        self.as_signal().add_subscriber(subscriber);
    }

    fn clear_subscribers(&self) {
        self.as_signal().clear_subscribers();
    }
}

impl<T: 'static> ToAnySource for SignalRead<T> {
    fn to_any_source(&self) -> AnySource {
        self.as_signal().to_any_source()
    }
}

impl<T: 'static> Track for SignalRead<T> {
    fn track(&self) {
        self.as_signal().track();
    }

    fn untrack(&self) {
        self.clear_subscribers();
    }
}

impl<T: 'static> Read for SignalRead<T> {
    type Value = T;

    fn read<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        self.as_signal().read(f)
    }

    fn try_read<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> Option<R> {
        self.as_signal().try_read(f)
    }
}

impl<T: Clone + 'static> Get for SignalRead<T> {
    type Value = T;

    fn get_untracked(&self) -> Self::Value {
        self.read(Clone::clone)
    }

    fn try_get_untracked(&self) -> Option<Self::Value> {
        self.try_read(Clone::clone)
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
        F: FnOnce(&Self::Value) -> R
    {
        self.try_read(f)
    }
}

impl<T: 'static> Dispose for SignalRead<T> {
    fn dispose(&self) { self.as_signal().dispose() }

    fn is_disposed(&self) -> bool {
        ReactiveStorage::is_removed(&self.node)
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
