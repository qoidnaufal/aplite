mod traits;
mod rw_signal;
mod arc_signal;
mod signal_read;
mod signal_write;

// use std::cell::RefCell;
use std::collections::HashMap;
// use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

pub use rw_signal::*;
pub use signal_read::*;
pub use signal_write::*;
pub use arc_signal::*;
pub use traits::*;

// thread_local! {
//     pub static REACTIVE_RUNTIME: RefCell<ReactiveRuntime> = RefCell::new(ReactiveRuntime::default());
// }

pub fn signal<T: 'static>(value: T) -> (SignalRead<T>, SignalWrite<T>) {
    RwSignal::new(value).split()
}

pub fn arc_signal<T>(value: T) -> ArcSignal<T>
where T: Send + Sync + 'static
{
    ArcSignal::new(value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignalId(u64);

impl SignalId {
    fn new() -> Self {
        static SIGNAL_ID: AtomicU64 = AtomicU64::new(0);
        Self(SIGNAL_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub(crate) struct AnySignal(Box<dyn Reactive>);

#[derive(Default)]
pub(crate) struct ReactiveRuntime {
    storage: HashMap<SignalId, AnySignal>,
    pending_update: Vec<SignalId>,
}

impl ReactiveRuntime {
    pub(crate) fn insert(&mut self, id: SignalId, signal: impl Into<AnySignal>) {
        self.storage.insert(id, signal.into());
    }
}
