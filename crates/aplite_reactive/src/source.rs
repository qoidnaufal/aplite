use aplite_storage::SlotId;
use crate::subscriber::{AnySubscriber, SubscriberSet, SubscriberStorage};

#[derive(Clone, Copy)]
pub(crate) struct AnySource(pub(crate) SlotId);

impl AnySource {
    pub(crate) fn new(id: SlotId) -> Self {
        Self(id)
    }

    pub(crate) fn update_if_necessary(&self) -> bool {
        SubscriberStorage::with(self.0, SubscriberSet::any_update).unwrap_or(false)
    }
}

pub(crate) trait Source {
    fn add_subscriber(&self, subscriber: AnySubscriber);
    fn clear_subscribers(&self);
}

impl Source for AnySource {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        SubscriberStorage::insert(self.0, subscriber);
    }

    fn clear_subscribers(&self) {
        SubscriberStorage::with_mut(self.0, |set| set.clear());
    }
}

impl PartialEq for AnySource {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
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
        *self
    }
}
