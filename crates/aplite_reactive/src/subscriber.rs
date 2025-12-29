use std::sync::{Arc, OnceLock, RwLock, Weak};
use aplite_storage::{SparseSet, SlotId};

use crate::graph::Observer;
use crate::reactive_traits::*;
use crate::source::AnySource;

/*
#########################################################
#
# SubscriberStorage
#
#########################################################
*/

#[derive(Default)]
pub(crate) struct SubscriberSet(pub(crate) Vec<AnySubscriber>);

impl SubscriberSet {
    pub(crate) fn any_update(&self) -> bool {
        self.0.iter().any(AnySubscriber::try_update)
    }
}

static SUBSCRIBER_STORAGE: OnceLock<RwLock<SubscriberStorage>> = OnceLock::new();

#[derive(Default)]
pub(crate) struct SubscriberStorage {
    storage: SparseSet<SlotId, SubscriberSet>,
    ids: Vec<SlotId>,
}

impl SubscriberStorage {
    fn write<'a>() -> std::sync::RwLockWriteGuard<'a, Self> {
        SUBSCRIBER_STORAGE.get_or_init(Default::default).write().unwrap()
    }

    pub(crate) fn insert(id: SlotId, subscriber: AnySubscriber) {
        let mut set = Self::write()
            .storage
            .get_or_insert_with(id, || SubscriberSet::default());

        if !set.0.contains(&subscriber) {
            set.0.push(subscriber);
        }
    }

    pub(crate) fn with_mut(id: SlotId, f: impl FnOnce(&mut Vec<AnySubscriber>)) {
        if let Some(set) = Self::write().storage.get_mut(id) {
            f(&mut set.0)
        }
    }

    pub(crate) fn remove(id: SlotId) {
        let mut lock = Self::write();
        if let Some(index) = lock.storage.get_data_index(id) {
            if let Some(last) = lock.ids.last().copied() {
                lock.ids.swap_remove(index);
                lock.storage.swap_remove(id, last);
            }
        }
    }

    fn read<'a>() -> std::sync::RwLockReadGuard<'a, Self> {
        SUBSCRIBER_STORAGE.get_or_init(Default::default).read().unwrap()
    }

    pub(crate) fn with<R>(id: SlotId, f: impl FnOnce(&SubscriberSet) -> R) -> Option<R> {
        Self::read().storage.get(id).map(f)
    }
}

unsafe impl Send for SubscriberStorage {}
unsafe impl Sync for SubscriberStorage {}

/*
#########################################################
#
# AnySubscriber
#
#########################################################
*/

pub(crate) struct AnySubscriber(pub(crate) Weak<dyn Subscriber>);

unsafe impl Send for AnySubscriber {}
unsafe impl Sync for AnySubscriber {}

impl AnySubscriber {
    pub(crate) fn new<T: Subscriber + 'static>(weak: Weak<T>) -> Self {
        Self(weak)
    }

    pub(crate) fn upgrade(&self) -> Option<Arc<dyn Subscriber>> {
        self.0.upgrade()
    }

    pub(crate) fn mark_dirty_owned(self) {
        self.mark_dirty();
    }

    pub(crate) fn needs_update(&self) -> bool {
        self.as_observer(|| self.try_update())
        // let prev = Observer::swap_observer(Some(self.clone()));
        // let res = self.try_update();
        // Observer::swap_observer(prev);
        // res
    }

    #[inline(always)]
    pub(crate) fn as_observer<R>(&self, f: impl FnOnce() -> R) -> R {
        let prev = Observer::swap_observer(Some(self.clone()));
        let res = f();
        Observer::swap_observer(prev);
        res
    }
}

/*
#########################################################
#
# Subscriber
#
#########################################################
*/

pub(crate) trait Subscriber: Reactive {
    fn add_source(&self, source: AnySource);
    fn clear_sources(&self);
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
    fn mark_dirty(&self) {
        if let Some(subscriber) = self.upgrade() {
            subscriber.mark_dirty();
        }
    }

    fn try_update(&self) -> bool {
        self.upgrade()
            .is_some_and(|subscriber| subscriber.try_update())
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
