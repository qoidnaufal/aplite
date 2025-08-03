use std::sync::{Arc, Weak};

use crate::reactive_traits::*;
use crate::source::AnySource;

pub(crate) struct AnySubscriber(pub(crate) Weak<dyn Subscriber>);

pub(crate) trait Subscriber: Notify {
    fn add_source(&self, source: AnySource);

    fn clear_source(&self);
}

pub(crate) trait ToAnySubscriber: Subscriber {
    fn to_any_subscriber(self) -> AnySubscriber;
}

impl ToAnySubscriber for AnySubscriber {
    fn to_any_subscriber(self) -> AnySubscriber {
        self.clone()
    }
}

impl AnySubscriber {
    pub(crate) fn new(inner: Arc<dyn Subscriber>) -> Self {
        Self(Arc::downgrade(&inner))
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
        write!(f, "AnySubscriber({:#x})", self.0.as_ptr() as *const usize as usize)
    }
}
