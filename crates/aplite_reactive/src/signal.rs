use std::sync::{Arc, RwLock};

use crate::signal_read::SignalRead;
use crate::graph::{ReactiveStorage, Node, Observer};
use crate::signal_write::SignalWrite;
use crate::reactive_traits::*;
use crate::source::*;
use crate::subscriber::*;

/*
#########################################################
#
# State
#
#########################################################
*/

pub(crate) type SignalNode<T> = Node<Arc<RwLock<SignalState<T>>>>;

pub(crate) struct SignalState<T> {
    pub(crate) value: T,
    pub(crate) subscribers: Subscribers,
}

unsafe impl<T> Send for SignalState<T> {}
unsafe impl<T> Sync for SignalState<T> {}

impl<T: 'static> SignalState<T> {
    pub(crate) fn new(value: T) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            value,
            subscribers: Subscribers::default(),
        }))
    }
}

impl<T: 'static> Reactive for RwLock<SignalState<T>> {
    fn mark_dirty(&self) {
        self.write().unwrap()
            .subscribers
            .mark_dirty()
    }

    fn try_update(&self) -> bool {
        false
    }
}

impl<T: 'static> Source for RwLock<SignalState<T>> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        self.write().unwrap()
            .subscribers
            .push(subscriber)
    }

    fn clear_subscribers(&self) {
        self.write().unwrap()
            .subscribers
            .clear()
    }
}

/*
#########################################################
#
# Signal
#
#########################################################
*/

pub struct Signal<T> {
    pub(crate) node: SignalNode<T>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> Copy for Signal<T> {}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        let node = ReactiveStorage::insert(SignalState::new(value));

        Self { node }
    }

    pub fn split(value: T) -> (SignalRead<T>, SignalWrite<T>) {
        Self::new(value).into_split()
    }

    pub fn into_split(self) -> (SignalRead<T>, SignalWrite<T>) {
        (SignalRead::new(self.node), SignalWrite::new(self.node))
    }
}

impl<T: 'static> Reactive for Signal<T> {
    #[inline(always)]
    fn mark_dirty(&self) {
        ReactiveStorage::with_downcast(&self.node, |state| {
            state.write().unwrap()
                .subscribers
                .mark_dirty()
        })
    }

    fn try_update(&self) -> bool {
        false
    }
}

impl<T: 'static> Source for Signal<T> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        ReactiveStorage::with_downcast(&self.node, |state| state.add_subscriber(subscriber))
    }

    fn clear_subscribers(&self) {
        ReactiveStorage::with_downcast(&self.node, |state| state.clear_subscribers())
    }
}

impl<T: 'static> ToAnySource for Signal<T> {
    fn to_any_source(&self) -> AnySource {
        ReactiveStorage::with_downcast(&self.node, AnySource::new)
    }
}

impl<T: 'static> Track for Signal<T> {
    fn track(&self) {
        Observer::with(|current| {
            if let Some(any_subscriber) = current {
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
        self.mark_dirty()
    }
}

impl<T: 'static> Read for Signal<T> {
    type Value = T;

    fn read<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        ReactiveStorage::with_downcast(&self.node, |state| {
            f(&state.read().unwrap().value)
        })
    }

    fn try_read<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> Option<R> {
        ReactiveStorage::try_with_downcast(&self.node, |state| {
            state.read().ok().map(|val| f(&val.value))
            // s.as_deref().map(|val| f(&val.value))
            // state.and_then(|ss| {
            //     let m = ss.read().ok();
            //     f(m.as_deref().map(|s| &s.value))
            // })
        })
    }
}

impl<T: Clone + 'static> Get for Signal<T> {
    type Value = T;

    fn get_untracked(&self) -> Self::Value {
        self.read(Clone::clone)
    }

    fn try_get_untracked(&self) -> Option<Self::Value> {
        self.try_read(Clone::clone)
    }
}

impl<T: 'static> With for Signal<T> {
    type Value = T;

    fn with_untracked<F, R>(&self, f: F) -> R where F: FnOnce(&Self::Value) -> R {
        self.read(f)
    }

    fn try_with_untracked<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Self::Value) -> R
    {
        self.try_read(f)
    }
}

impl<T: 'static> Write for Signal<T> {
    type Value = T;

    fn write(&self, f: impl FnOnce(&mut Self::Value)) {
        ReactiveStorage::with_downcast(&self.node, |state| {
            f(&mut state.write().unwrap().value)
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
        ReactiveStorage::remove(self.node);
    }

    fn is_disposed(&self) -> bool {
        ReactiveStorage::is_removed(&self.node)
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

/*
#########################################################
#
# Test
#
#########################################################
*/

#[cfg(test)]
mod signal_test {
    use super::*;

    #[test]
    fn signal() {
        let (counter, set_counter) = Signal::split(0i32);

        set_counter.update(|num| *num += 1);
        assert_eq!(counter.get(), 1);

        set_counter.set(-69);
        assert_eq!(counter.get(), -69);

        let r = counter.try_with(ToString::to_string);
        assert!(r.is_some());
        assert_eq!(r.unwrap().parse(), Ok(-69));
    }

    #[test]
    fn derive() {
        let rw = Signal::new(0i32);
        let (counter, set_counter) = Signal::split(0i32);

        set_counter.set(69);
        rw.update(|num| *num = counter.get());
        assert_eq!(rw.get(), 69);
    }

    #[test]
    #[should_panic]
    fn dispose() {
        let (num, set_num) = Signal::split(0i32);
        let double = || num.get() * 2;

        set_num.set(1);
        assert_eq!(double(), 2);

        num.dispose();

        set_num.set(2);
        assert_eq!(double(), 2);
    }
}
