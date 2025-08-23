// use std::sync::{Arc, Weak};
use crate::subscriber::AnySubscriber;
use crate::reactive_traits::*;

// pub(crate) struct AnySource(pub(crate) Weak<dyn Source>);

// impl AnySource {
//     pub(crate) fn new(inner: Arc<dyn Source>) -> Self {
//         Self(Arc::downgrade(&inner))
//     }

//     pub(crate) fn upgrade(&self) -> Option<Arc<dyn Source>> {
//         self.0.upgrade()
//     }
// }

pub(crate) trait Source: Track + Notify {
    fn add_subscriber(&self, subscriber: AnySubscriber);
    fn clear_subscribers(&self);
}

// impl Source for AnySource {
//     fn add_subscriber(&self, subscriber: AnySubscriber) {
//         if let Some(any_source) = self.0.upgrade() {
//             any_source.add_subscriber(subscriber);
//         }
//     }

//     fn clear_subscribers(&self) {
//         if let Some(source) = self.upgrade() {
//             source.clear_subscribers();
//         }
//     }
// }

// impl Track for AnySource {
//     fn track(&self) {
//         if let Some(source) = self.upgrade() {
//             source.track()
//         }
//     }

//     fn untrack(&self) {
//         if let Some(source) = self.upgrade() {
//             source.untrack();
//         }
//     }
// }

// impl Notify for AnySource {
//     fn notify(&self) {
//         if let Some(source) = self.0.upgrade() {
//             source.notify();
//         }
//     }
// }

// impl Clone for AnySource {
//     fn clone(&self) -> Self {
//         Self(Weak::clone(&self.0))
//     }
// }

// impl PartialEq for AnySource {
//     fn eq(&self, other: &Self) -> bool {
//         Weak::ptr_eq(&self.0, &other.0)
//     }
// }

// impl std::fmt::Debug for AnySource {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "AnySource({:#x})", &(Weak::as_ptr(&self.0).addr()))
//     }
// }

// pub(crate) trait ToAnySource: Source {
//     fn to_any_source(self) -> AnySource;
// }

// impl ToAnySource for AnySource {
//     fn to_any_source(self) -> AnySource {
//         self.clone()
//     }
// }
