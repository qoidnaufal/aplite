use std::sync::{Weak, Arc};

use crate::subscriber::AnySubscriber;
use crate::reactive_traits::*;

pub(crate) struct AnySource(pub(crate) Weak<dyn Source>);

impl AnySource {
    pub(crate) fn new<T: Source + 'static>(arc: &Arc<T>) -> Self {
        let weak: Weak<T> = Arc::downgrade(arc);
        Self(weak)
    }

    pub(crate) fn upgrade(&self) -> Option<Arc<dyn Source>> {
        self.0.upgrade()
    }

    pub(crate) fn update_if_necessary(&self) -> bool {
        self.upgrade()
            .map(|source| source.try_update())
            .unwrap_or(false)
    }
}

impl Clone for AnySource {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

impl Reactive for AnySource {
    fn mark_dirty(&self) {
        if let Some(source) = self.upgrade() {
            source.mark_dirty();
        }
    }

    fn try_update(&self) -> bool {
        self.update_if_necessary()
    }
}

pub(crate) trait Source: Reactive {
    fn add_subscriber(&self, subscriber: AnySubscriber);
    fn clear_subscribers(&self);
}

impl Source for AnySource {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        if let Some(source) = self.upgrade() {
            source.add_subscriber(subscriber);
        }
    }

    fn clear_subscribers(&self) {
        if let Some(source) = self.upgrade() {
            source.clear_subscribers();
        }
    }
}

impl PartialEq for AnySource {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for AnySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnySource({:?})", self.0)
    }
}

pub(crate) trait ToAnySource: Source {
    fn to_any_source(&self) -> AnySource;
}

impl ToAnySource for AnySource {
    fn to_any_source(&self) -> AnySource {
        self.clone()
    }
}
