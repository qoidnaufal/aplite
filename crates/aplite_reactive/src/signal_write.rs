use crate::graph::ReactiveStorage;
use crate::signal::{Signal, SignalNode};
use crate::signal_read::SignalRead;
use crate::reactive_traits::*;

pub struct SignalWrite<T> {
    pub(crate) node: SignalNode<T>,
}

impl<T> Clone for SignalWrite<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> Copy for SignalWrite<T> {}

impl<T: 'static> SignalWrite<T> {
    pub(crate) fn new(node: SignalNode<T>) -> Self {
        Self { node }
    }

    #[inline(always)]
    pub fn as_signal(&self) -> Signal<T> {
        Signal { node: self.node }
    }
}

impl<T: 'static> Notify for SignalWrite<T> {
    fn notify(&self) {
        self.as_signal().notify()
    }
}

impl<T: 'static> Write for SignalWrite<T> {
    type Value = T;

    fn write(&self, f: impl FnOnce(&mut Self::Value)) {
        self.as_signal().write(f);
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
        ReactiveStorage::is_removed(&self.node)
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
