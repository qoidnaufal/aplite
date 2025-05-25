use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::effect::Effect;
// use crate::signal::Signal;
use crate::RwSignal;

thread_local! {
    pub static RUNTIME: Runtime = Runtime::new();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReactiveId(u64);

impl ReactiveId {
    pub(crate) fn new() -> Self {
        static REACTIVE_ID: AtomicU64 = AtomicU64::new(0);
        Self(REACTIVE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EffectId(u64);

impl EffectId {
    pub(crate) fn new() -> Self {
        static EFFECT_ID: AtomicU64 = AtomicU64::new(0);
        Self(EFFECT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

type ReactiveValue = Box<dyn Any>;

pub(crate) struct Runtime {
    pub(crate) storage: RefCell<HashMap<ReactiveId, ReactiveValue>>,
    pub(crate) current_effect: RefCell<Option<EffectId>>,
    pub(crate) subscribers: RefCell<HashMap<ReactiveId, HashSet<EffectId>>>,
    pub(crate) effect_storage: RefCell<HashMap<EffectId, Effect>>,
}

impl Runtime {
    pub(crate) fn new() -> Self {
        Self {
            storage: RefCell::new(HashMap::new()),
            current_effect: RefCell::new(None),
            subscribers: RefCell::new(HashMap::new()),
            effect_storage: RefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn create_rw_signal<T: Any + 'static>(&self, id: ReactiveId, value: T) -> RwSignal<T> {
        self.storage.borrow_mut().insert(id, Box::new(RefCell::new(value)));
        RwSignal { id, phantom: PhantomData }
    }

    pub(crate) fn create_effect<F: Fn() + 'static>(&self, f: F) {
        let id = EffectId::new();
        let effect = Effect {
            f: Box::new(f),
        };
        self.effect_storage.borrow_mut().insert(id, effect);
        self.run_effect(&id);
    }

    fn run_effect(&self, effect_id: &EffectId) {
        let pref_effect = self.current_effect.take();
        *self.current_effect.borrow_mut() = Some(*effect_id);
        if let Some(effect) = self.effect_storage.borrow().get(effect_id) {
            effect.run();
        }
        *self.current_effect.borrow_mut() = pref_effect;
    }

    pub(crate) fn add_subscriber(&self, id: ReactiveId) {
        let current = self.current_effect.borrow();
        if let Some(effect_id) = current.as_ref() {
            let mut subscribers = self.subscribers.borrow_mut();
            subscribers.entry(id).or_default().insert(*effect_id);
        }
    }

    pub(crate) fn notify_subscribers(&self, id: ReactiveId) {
        let subs = {
            let subs = self.subscribers.borrow();
            subs.get(&id).cloned()
        };
        if let Some(subscribers) = subs {
            subscribers
                .iter()
                .for_each(|effect_id| self.run_effect(effect_id));
        }
    }
}

#[cfg(test)]
mod signal_test {
    use crate::signal::Signal;
    use crate::traits::*;

    #[test]
    fn signal_test() {
        let (counter, set_counter) = Signal::new(0i32);

        set_counter.update(|num| *num += 1);
        assert_eq!(counter.get(), 1);

        set_counter.set(-69);
        assert_eq!(counter.get(), -69);

        let r = counter.with(|num| num.to_string());
        assert_eq!(r.parse(), Ok(-69));
    }
}
