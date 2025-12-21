use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};
use aplite_storage::{SparseSet, SlotId};

use crate::graph::Graph;
use crate::reactive_traits::*;
use crate::source::AnySource;

/*
#########################################################
#
# Core types
#
#########################################################
*/

#[derive(Default)]
pub(crate) struct SubscriberSet(pub(crate) Vec<AnySubscriber>);

pub(crate) struct AnySubscriber(pub(crate) Weak<dyn Subscriber>);

/*
#########################################################
#
# SubscriberStorage
#
#########################################################
*/

static SUBSCRIBER_STORAGE: OnceLock<RwLock<SubscriberStorage>> = OnceLock::new();

#[derive(Default)]
pub(crate) struct SubscriberStorage {
    storage: SparseSet<SlotId, SubscriberSet>,
    ids: Vec<SlotId>,
}

impl SubscriberStorage {
    fn read<'a>() -> RwLockReadGuard<'a, Self> {
        SUBSCRIBER_STORAGE.get_or_init(Default::default).read().unwrap()
    }

    fn write<'a>() -> RwLockWriteGuard<'a, Self> {
        SUBSCRIBER_STORAGE.get_or_init(Default::default).write().unwrap()
    }

    pub(crate) fn insert(id: SlotId, subscriber: AnySubscriber) {
        let mut lock = Self::write();
        if let Some(subscriber_set) = lock.storage.get_mut(id) {
            if !subscriber_set.0.contains(&subscriber) {
                subscriber_set.0.push(subscriber);
            }
        } else {
            let mut set = lock.storage.insert(id, SubscriberSet::default());
            set.as_mut().0.push(subscriber);
        }
    }

    pub(crate) fn with<R>(id: SlotId, f: impl FnOnce(&SubscriberSet) -> R) -> Option<R> {
        Self::read().storage.get(id).map(f)
    }

    pub(crate) fn with_mut(id: SlotId, f: impl FnOnce(&mut Vec<AnySubscriber>)) {
        if let Some(set) = Self::write().storage.get_mut(id) {
            f(&mut set.0)
        }
    }

    pub(crate) fn remove(id: SlotId) {
        let mut lock = Self::write();
        if let Some(last) = lock.ids.last().copied() {
            let index = lock.storage.get_data_index(id).unwrap();
            lock.ids.swap_remove(index);
            lock.storage.swap_remove(id, last);
        }
    }
}

unsafe impl Send for SubscriberStorage {}
unsafe impl Sync for SubscriberStorage {}

/*
#########################################################
#
# Subscriber
#
#########################################################
*/

pub(crate) trait Subscriber: Notify + Reactive {
    fn add_source(&self, source: AnySource);
    fn clear_sources(&self);
}

impl AnySubscriber {
    pub(crate) fn new<T: Subscriber + 'static>(weak: Weak<T>) -> Self {
        Self(weak)
    }

    pub(crate) fn upgrade(&self) -> Option<Arc<dyn Subscriber>> {
        self.0.upgrade()
    }

    pub(crate) fn notify_owned(self) {
        self.notify();
    }

    pub(crate) fn try_update(&self) -> bool {
        let prev = Graph::set_observer(Some(self.clone()));
        let res = self.update_if_necessary();
        Graph::set_observer(prev);
        res
    }

    pub(crate) fn as_observer<R>(&self, f: impl FnOnce() -> R) -> R {
        let prev = Graph::set_observer(Some(self.clone()));
        let res = f();
        Graph::set_observer(prev);
        res
    }
}

impl Subscriber for AnySubscriber {
    fn add_source(&self, source: AnySource) {
        if let Some(subscriber) = self.upgrade() {
            subscriber.add_source(source);
        }
    }

    fn clear_sources(&self) {
        if let Some(subscriber) = self.upgrade() {
            subscriber.clear_sources();
        }
    }
}

impl Reactive for AnySubscriber {
    fn update_if_necessary(&self) -> bool {
        self.upgrade()
            .is_some_and(|subscriber| subscriber.update_if_necessary())
    }
}

impl Notify for AnySubscriber {
    fn notify(&self) {
        if let Some(subscriber) = self.upgrade() {
            subscriber.notify();
        }
    }
}

impl Clone for AnySubscriber {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

impl PartialEq for AnySubscriber {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl PartialEq<&AnySubscriber> for AnySubscriber {
    fn eq(&self, other: &&AnySubscriber) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl PartialEq<AnySubscriber> for &AnySubscriber {
    fn eq(&self, other: &AnySubscriber) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for AnySubscriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnySubscriber({:#x})", self.0.as_ptr().addr())
    }
}

/*
#########################################################
#
# ToAnySubscriber
#
#########################################################
*/

pub(crate) trait ToAnySubscriber: Subscriber {
    fn to_any_subscriber(&self) -> AnySubscriber;
}

impl ToAnySubscriber for AnySubscriber {
    fn to_any_subscriber(&self) -> AnySubscriber {
        self.clone()
    }
}
