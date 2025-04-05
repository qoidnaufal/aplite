use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicU64, Ordering};

thread_local! {
    pub static SIGNAL_RUNTIME: RefCell<SignalRuntime> = RefCell::new(SignalRuntime::default());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SignalId(u64);

impl SignalId {
    fn new() -> Self {
        static SIGNAL_ID: AtomicU64 = AtomicU64::new(0);
        Self(SIGNAL_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone)]
pub struct Signal<T> {
    id: SignalId,
    value: Arc<Mutex<T>>,
}

impl<T> PartialEq for Signal<T>
where
    T: PartialEq + Eq + Clone + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T> Eq for Signal<T> where T: PartialEq + Eq + Clone + 'static {}

impl<T: Clone + 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        let signal = Self {
            id: SignalId::new(),
            value: Arc::new(Mutex::new(value)),
        };
        SIGNAL_RUNTIME.with_borrow_mut(|rt| rt.insert(signal.id(), &signal));
        signal
    }

    pub(crate) fn id(&self) -> SignalId { self.id }

    pub fn borrow(&self) -> MutexGuard<'_, T> {
        self.value.lock().unwrap()
    }

    pub fn get(&self) -> T { (*self.borrow()).clone() }

    pub fn set<F>(&self, f: F)
    where
        F: FnOnce(&mut T) + 'static,
    {
        f(&mut self.borrow());
        SIGNAL_RUNTIME.with_borrow_mut(|rt| rt.push_update(self.id()))
    }
}

unsafe impl<T> Send for Signal<T> {}
unsafe impl<T> Sync for Signal<T> {}

pub(crate) struct AnySignal {
    id: SignalId,
    value: Arc<dyn Any>,
}

impl AnySignal {
    fn id(&self) -> SignalId { self.id }

    fn cast<T: Clone + 'static>(&self) -> T {
        self.value.downcast_ref::<T>().unwrap().clone()
    }
}

impl<T: Clone + 'static> From<Signal<T>> for AnySignal {
    fn from(signal: Signal<T>) -> Self {
        Self {
            id: signal.id(),
            value: Arc::new(signal.get()),
        }
    }
}

impl<T: Clone + 'static> From<&Signal<T>> for AnySignal {
    fn from(signal: &Signal<T>) -> Self {
        Self {
            id: signal.id(),
            value: Arc::new(signal.get()),
        }
    }
}

impl<T: Clone + 'static> From<AnySignal> for Signal<T> {
    fn from(any: AnySignal) -> Self {
        Self {
            id: any.id(),
            value: Arc::new(Mutex::new(any.cast())),
        }
    }
}

impl<T: Clone + 'static> From<&AnySignal> for Signal<T> {
    fn from(any: &AnySignal) -> Self {
        Self {
            id: any.id(),
            value: Arc::new(Mutex::new(any.cast())),
        }
    }
}

#[derive(Default)]
pub struct SignalRuntime {
    storage: HashMap<SignalId, AnySignal>,
    updated: Vec<SignalId>,
}

impl SignalRuntime {
    pub fn insert(&mut self, id: SignalId, signal: impl Into<AnySignal>) {
        self.storage.insert(id, signal.into());
    }

    pub(crate) fn get<T: Clone + 'static>(&self, id: &SignalId) -> Option<Signal<T>> {
        self.storage.get(id).map(|any| any.into())
    }

    fn push_update(&mut self, id: SignalId) {
        self.updated.push(id);
    }
}
