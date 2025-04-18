mod traits;
mod signal;

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

pub use signal::*;
pub use traits::*;

// thread_local! {
//     pub static REACTIVE_RUNTIME: RefCell<ReactiveRuntime> = RefCell::new(ReactiveRuntime::default());
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignalId(u64);

impl SignalId {
    fn new() -> Self {
        static SIGNAL_ID: AtomicU64 = AtomicU64::new(0);
        Self(SIGNAL_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub(crate) struct AnySignal {
    id: SignalId,
    value: Rc<RefCell<dyn Any>>,
}

impl AnySignal {
    fn id(&self) -> SignalId { self.id }

    fn cast<T: Clone + 'static>(&self) -> T {
        self.value.borrow().downcast_ref::<T>().unwrap().clone()
    }
}

impl<T: Clone + 'static> From<Signal<T>> for AnySignal {
    fn from(signal: Signal<T>) -> Self {
        Self {
            id: signal.id(),
            value: Rc::new(RefCell::new(signal.get())),
        }
    }
}

impl<T: Clone + 'static> From<&Signal<T>> for AnySignal {
    fn from(signal: &Signal<T>) -> Self {
        Self {
            id: signal.id(),
            value: Rc::new(RefCell::new(signal.get())),
        }
    }
}

impl<T: Clone + 'static> From<ArcSignal<T>> for AnySignal {
    fn from(arc_signal: ArcSignal<T>) -> Self {
        Self {
            id: arc_signal.id(),
            value: Rc::new(RefCell::new(arc_signal.get())),
        }
    }
}

impl<T: Clone + 'static> From<&ArcSignal<T>> for AnySignal {
    fn from(arc_signal: &ArcSignal<T>) -> Self {
        Self {
            id: arc_signal.id(),
            value: Rc::new(RefCell::new(arc_signal.get())),
        }
    }
}

#[derive(Default)]
pub(crate) struct ReactiveRuntime {
    storage: HashMap<SignalId, AnySignal>,
    pending_update: Vec<SignalId>,
}

impl ReactiveRuntime {
    pub(crate) fn insert(&mut self, id: SignalId, signal: impl Into<AnySignal>) {
        self.storage.insert(id, signal.into());
    }

    pub(crate) fn get<T: Clone + 'static>(&self, id: &SignalId) -> Option<Signal<T>> {
        self.storage.get(id).map(|any| any.into())
    }

    pub(crate) fn push_update(&mut self, id: SignalId) {
        self.pending_update.push(id);
    }
}
