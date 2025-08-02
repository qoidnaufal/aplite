use std::sync::{Arc, Weak};
use crate::graph::ReactiveId;

pub(crate) struct AnySubscriber(Arc<dyn Subscriber>);

pub(crate) struct WeakSubscriber(Weak<dyn Subscriber>);

pub(crate) trait Subscriber {
    fn notify(&self);

    fn add_source(&self, source: ReactiveId); // find a way to use dyn Source here

    fn clear_source(&self);
}

pub(crate) trait ToAnySubscriber {
    fn to_any_subscriber(self) -> AnySubscriber;
}

impl AnySubscriber {
    pub(crate) fn new<T: Subscriber + 'static>(inner: T) -> Self {
        Self(Arc::new(inner))
    }

    pub(crate) fn downgrade(&self) -> WeakSubscriber {
        WeakSubscriber(Arc::downgrade(&self.0))
    }
}

impl WeakSubscriber {
    pub(crate) fn upgrade(&self) -> Option<AnySubscriber> {
        self.0
            .upgrade()
            .map(|s| AnySubscriber(s))
    }
}

impl Subscriber for AnySubscriber {
    fn notify(&self) {
        self.0.notify();
    }

    fn add_source(&self, source: ReactiveId) {
        self.0.add_source(source);
    }

    fn clear_source(&self) {
        self.0.clear_source();
    }
}

impl Subscriber for WeakSubscriber {
    fn notify(&self) {
        if let Some(any_subscriber) = self.upgrade() {
            any_subscriber.notify();
        }
    }

    fn add_source(&self, source: ReactiveId) {
        if let Some(any_subscriber) = self.upgrade() {
            any_subscriber.add_source(source);
        }
    }

    fn clear_source(&self) {
        if let Some(any_subscriber) = self.upgrade() {
            any_subscriber.0.clear_source();
        }
    }
}

impl Clone for AnySubscriber {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl PartialEq for AnySubscriber {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for AnySubscriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", &(Arc::as_ptr(&self.0) as *const usize as usize))
    }
}

impl Clone for WeakSubscriber {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

impl PartialEq for WeakSubscriber {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for WeakSubscriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", self.0.as_ptr() as *const usize as usize)
    }
}
