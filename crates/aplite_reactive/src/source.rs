use crate::subscriber::{AnySubscriber, SubscriberStorage};
use aplite_storage::Entity;

pub(crate) struct AnySource(pub(crate) Entity);

impl AnySource {
    pub(crate) fn new(id: Entity) -> Self {
        Self(id)
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
        SubscriberStorage::with_mut(&self.0, |set| set.clear());
    }
}

impl Clone for AnySource {
    fn clone(&self) -> Self {
        Self(self.0)
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

// impl ToAnySource for AnySource {
//     fn to_any_source(&self) -> AnySource {
//         self.clone()
//     }
// }
