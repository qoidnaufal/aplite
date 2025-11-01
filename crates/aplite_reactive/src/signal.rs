use crate::signal_read::SignalRead;
use crate::graph::{Node, Graph};
use crate::signal_write::SignalWrite;
use crate::stored_value::Value;
use crate::reactive_traits::*;
use crate::source::*;
use crate::subscriber::*;

pub(crate) type SignalNode<T> = Node<Value<T>>;

pub struct Signal<T> {
    pub(crate) node: SignalNode<T>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> Copy for Signal<T> {}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        let node = Graph::insert(Value::new(value));

        Self { node }
    }

    pub fn split(value: T) -> (SignalRead<T>, SignalWrite<T>) {
        Self::new(value).into_split()
    }

    pub fn into_split(self) -> (SignalRead<T>, SignalWrite<T>) {
        (SignalRead::new(self.node), SignalWrite::new(self.node))
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
        SubscriberStorage::insert(self.node.id, subscriber);
    }

    fn clear_subscribers(&self) {
        SubscriberStorage::with_mut(&self.node.id, |set| set.clear());
    }
}

impl<T: 'static> ToAnySource for Signal<T> {
    fn to_any_source(&self) -> AnySource {
        AnySource::new(self.node.id)
    }
}

impl<T: 'static> Track for Signal<T> {
    fn track(&self) {
        Graph::with(|graph| {
            if let Some(any_subscriber) = graph.current.as_ref() {
                any_subscriber.add_source(self.to_any_source());
                self.add_subscriber(any_subscriber.clone());
            }
        })
    }

    fn untrack(&self) {
        self.clear_subscribers();
    }
}

impl<T: 'static> Notify for Signal<T> {
    fn notify(&self) {
        SubscriberStorage::with_mut(&self.node.id, |set| {
            set.drain(..)
                .for_each(|any_subscriber| any_subscriber.notify());
        });
    }
}

impl<T: 'static> Read for Signal<T> {
    type Value = T;

    fn read<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        Graph::with_downcast(&self.node, |value| {
            f(&value.read().unwrap())
        })
    }

    fn try_read<R, F: FnOnce(Option<&Self::Value>) -> Option<R>>(&self, f: F) -> Option<R> {
        Graph::try_with_downcast(&self.node, |value| {
            let guard = value.and_then(|val| val.read().ok());
            f(guard.as_deref())
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
        Graph::with_downcast(&self.node, |value| {
            f(&mut value.write().unwrap())
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
        Graph::remove(&self.node);
        SubscriberStorage::remove(&self.node.id);
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
