use std::sync::{Arc, Weak, OnceLock, RwLock, RwLockWriteGuard};
use std::collections::HashMap;

use aplite_storage::Entity;

use crate::reactive_traits::*;
use crate::source::AnySource;

static SUBSCRIBER_STORAGE: OnceLock<RwLock<StorageInner>> = OnceLock::new();

#[derive(Default)]
struct StorageInner {
    storage: HashMap<Entity, SubscriberSet>,
}

pub(crate) struct SubscriberStorage;

impl SubscriberStorage {
    // fn read<'a>() -> RwLockReadGuard<'a, StorageInner> {
    //     SUBSCRIBER_STORAGE.get_or_init(Default::default).read().unwrap()
    // }

    fn write<'a>() -> RwLockWriteGuard<'a, StorageInner> {
        SUBSCRIBER_STORAGE.get_or_init(Default::default).write().unwrap()
    }

    pub(crate) fn insert(id: Entity, subscriber: AnySubscriber) {
        Self::write()
            .storage
            .entry(id)
            .or_insert(SubscriberSet::default()).0
            .push(subscriber);
    }

    pub(crate) fn with_mut(id: &Entity, f: impl FnOnce(&mut Vec<AnySubscriber>)) {
        let mut lock = Self::write();
        if let Some(set) = lock.storage.get_mut(id) {
            f(&mut set.0)
        }
    }

    pub(crate) fn remove(id: &Entity) {
        let mut lock = Self::write();
        lock.storage.remove(id);
    }
}

unsafe impl Send for StorageInner {}
unsafe impl Sync for StorageInner {}

#[derive(Default)]
pub(crate) struct SubscriberSet(pub(crate) Vec<AnySubscriber>);

pub(crate) struct AnySubscriber(pub(crate) Weak<dyn Subscriber>);

pub(crate) trait Subscriber: Notify {
    fn add_source(&self, source: AnySource);
    fn clear_source(&self);
    fn source_count(&self) -> usize;
}

impl AnySubscriber {
    pub(crate) fn new<T: Subscriber + 'static>(inner: Weak<T>) -> Self {
        Self(inner)
    }

    pub(crate) fn upgrade(&self) -> Option<Arc<dyn Subscriber>> {
        self.0.upgrade()
    }
}

impl Subscriber for AnySubscriber {
    fn add_source(&self, source: AnySource) {
        if let Some(subscriber) = self.upgrade() {
            subscriber.add_source(source);
        }
    }

    fn clear_source(&self) {
        if let Some(subscriber) = self.upgrade() {
            subscriber.clear_source();
        }
    }

    fn source_count(&self) -> usize {
        self.upgrade()
            .map(|any_subscriber| any_subscriber.source_count())
            .unwrap_or_default()
    }
}

impl Notify for AnySubscriber {
    fn notify(&self) {
        if let Some(subscriber) = self.upgrade() {
            subscriber.notify();
        }
    }
}

impl Track for AnySubscriber {
    fn track(&self) {}
    fn untrack(&self) {}
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

pub(crate) trait ToAnySubscriber: Subscriber {
    fn to_any_subscriber(&self) -> AnySubscriber;
}

impl ToAnySubscriber for AnySubscriber {
    fn to_any_subscriber(&self) -> AnySubscriber {
        self.clone()
    }
}
