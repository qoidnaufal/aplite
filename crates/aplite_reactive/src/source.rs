use std::rc::{Rc, Weak};
use crate::subscriber::{WeakSubscriber, AnySubscriber};

pub(crate) struct AnySource(Rc<dyn Source>);

pub(crate) struct WeakSource(Weak<dyn Source>);

pub(crate) trait Source {
    fn get_subscribers(&self) -> Option<Vec<AnySubscriber>>;
    fn add_subscriber(&self, subscriber: WeakSubscriber);
    fn clear_subscribers(&self);
}

pub(crate) trait ToAnySource {
    fn to_any_source(self) -> AnySource;
}

impl Source for AnySource {
    fn get_subscribers(&self) -> Option<Vec<AnySubscriber>> {
        self.0.get_subscribers()
    }

    fn add_subscriber(&self, subscriber: WeakSubscriber) {
        self.0.add_subscriber(subscriber);
    }

    fn clear_subscribers(&self) {
        self.0.clear_subscribers();
    }
}

impl Source for WeakSource {
    fn get_subscribers(&self) -> Option<Vec<AnySubscriber>> {
        self.upgrade()
            .and_then(|any_source| any_source.get_subscribers())
    }

    fn add_subscriber(&self, subscriber: WeakSubscriber) {
        if let Some(any_source) = self.upgrade() {
            any_source.add_subscriber(subscriber);
        }
    }

    fn clear_subscribers(&self) {
        if let Some(any_source) = self.upgrade() {
            any_source.clear_subscribers();
        }
    }
}

impl AnySource {
    pub(crate) fn new<T: Source + 'static>(source: T) -> Self {
        Self(Rc::new(source))
    }

    pub(crate) fn downgrade(&self) -> WeakSource {
        WeakSource(Rc::downgrade(&self.0))
    }
}

impl WeakSource {
    pub(crate) fn upgrade(&self) -> Option<AnySource> {
        self.0
            .upgrade()
            .map(|source| AnySource(source))
    }
}

impl Clone for AnySource {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl PartialEq for AnySource {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for AnySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnySource")
            .field("address", &Rc::as_ptr(&self.0).addr())
            .finish()
    }
}

impl Clone for WeakSource {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

impl PartialEq for WeakSource {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for WeakSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnySource")
            .field("address", &Weak::as_ptr(&self.0).addr())
            .finish()
    }
}
