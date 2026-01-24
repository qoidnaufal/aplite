use std::sync::{Arc, Weak};

use crate::graph::Observer;
use crate::reactive_traits::*;
use crate::source::AnySource;

/*
#########################################################
#
# Subscribers
#
#########################################################
*/

#[derive(Default)]
// TODO: Conside using SparseSet to make removal of a single AnySubscriber much more convenient
pub(crate) struct Subscribers(pub(crate) Vec<AnySubscriber>);

impl Subscribers {
    pub(crate) fn mark_dirty(&mut self) {
        let subs = std::mem::take(&mut self.0);
        subs.iter().for_each(AnySubscriber::mark_dirty);
    }

    pub(crate) fn push(&mut self, subscriber: AnySubscriber) {
        if !self.0.contains(&subscriber) {
            self.0.push(subscriber);
        }
    }

    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }
}

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
    pub(crate) fn from_weak<T: Subscriber + 'static>(weak: Weak<T>) -> Self {
        Self(weak)
    }

    pub(crate) fn new<T: Subscriber + 'static>(arc: &Arc<T>) -> Self {
        let weak: Weak<T> = Arc::downgrade(arc);
        Self(weak)
    }

    pub(crate) fn empty<T: Subscriber + 'static>() -> Self {
        let weak: Weak<T> = Weak::new();
        Self(weak)
    }

    fn upgrade(&self) -> Option<Arc<dyn Subscriber>> {
        self.0.upgrade()
    }

    pub(crate) fn needs_update(&self) -> bool {
        self.as_observer(|| self.try_update())
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
