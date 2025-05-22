mod traits;
mod rw_signal;
mod arc_signal;
mod signal_read;
mod signal_write;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

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
pub struct ReactiveId(u64);

impl ReactiveId {
    fn new() -> Self {
        static SIGNAL_ID: AtomicU64 = AtomicU64::new(0);
        Self(SIGNAL_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub(crate) struct AnySignal(Box<dyn Reactive>);

impl std::ops::Deref for AnySignal {
    type Target = Box<dyn Reactive>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) struct ReactiveGraph {
    storage: HashMap<ReactiveId, AnySignal>,
    observer: Arc<std::sync::mpsc::Sender<()>>,
    subscriber: Arc<std::sync::mpsc::Receiver<()>>,
    pending_update: Vec<ReactiveId>,
}

impl ReactiveGraph {
    pub(crate) fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            storage: HashMap::new(),
            observer: Arc::new(tx),
            subscriber: Arc::new(rx),
            pending_update: Vec::new(),
        }
    }

    pub(crate) fn get_observer(&self) -> Arc<std::sync::mpsc::Sender<()>> {
        Arc::clone(&self.observer)
    }

    pub(crate) fn insert(&mut self, id: ReactiveId, signal: impl Reactive + 'static) {
        self.storage.insert(id, AnySignal(Box::new(signal)));
    }
}
