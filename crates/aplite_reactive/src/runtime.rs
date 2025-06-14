use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::atomic::AtomicU64;

use crate::effect::Effect;
use crate::signal::Signal;
use crate::{AnySubscriber, ToAnySubscriber, RwSignal};

thread_local! {
    pub static RUNTIME: ReactiveGraph = ReactiveGraph::new();
}

// -------------------------------------------------------- //
// -------------------------------------------------------- //
//                                                          //
//                            Id                            //
//                                                          //
// -------------------------------------------------------- //
// -------------------------------------------------------- //

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReactiveId(u64);

impl ReactiveId {
    pub(crate) fn new() -> Self {
        static REACTIVE_ID: AtomicU64 = AtomicU64::new(0);
        Self(REACTIVE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct EffectId(u64);

impl EffectId {
    pub(crate) fn new() -> Self {
        static EFFECT_ID: AtomicU64 = AtomicU64::new(0);
        Self(EFFECT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

// -------------------------------------------------------- //
// -------------------------------------------------------- //
//                                                          //
//                          Graph                           //
//                                                          //
// -------------------------------------------------------- //
// -------------------------------------------------------- //

type SignalStorage = HashMap<ReactiveId, Signal>;
type EffectStorage = HashMap<EffectId, AnySubscriber>;
type Observers = HashMap<ReactiveId, HashSet<EffectId>>;

pub(crate) struct ReactiveGraph {
    pub(crate) signals: RefCell<SignalStorage>,
    pub(crate) current: RefCell<Option<EffectId>>,
    pub(crate) observers: RefCell<Observers>,
    pub(crate) subscribers: RefCell<EffectStorage>,
}

impl ReactiveGraph {
    pub(crate) fn new() -> Self {
        Self {
            signals: Default::default(),
            current: Default::default(),
            observers: Default::default(),
            subscribers: Default::default(),
        }
    }

    pub(crate) fn create_rw_signal<T: Any + 'static>(&self, value: T) -> RwSignal<T> {
        let id = ReactiveId::new();
        self.signals.borrow_mut().insert(id, Signal::stored(value));
        RwSignal { id, phantom: PhantomData }
    }

    pub(crate) fn add_subscriber(&self, id: ReactiveId) {
        let current = self.current.borrow();
        if let Some(effect_id) = current.as_ref() {
            let mut subscribers = self.observers.borrow_mut();
            subscribers.entry(id).or_default().insert(*effect_id);
        }
    }

    pub(crate) fn notify_subscribers(&self, id: ReactiveId) {
        let observers = self.observers.borrow();
        let maybe_subs = observers.get(&id).cloned();
        drop(observers);
        if let Some(subscribers) = maybe_subs {
            subscribers
                .iter()
                .for_each(|effect_id| self.run_effect(effect_id));
        }
    }

    fn run_effect(&self, effect_id: &EffectId) {
        let pref_effect = self.current.take();
        *self.current.borrow_mut() = Some(*effect_id);
        if let Some(effect) = self.subscribers
            .borrow_mut()
            .get_mut(&effect_id) {
                effect.run();
            }
        *self.current.borrow_mut() = pref_effect;
    }

    pub(crate) fn create_effect<F, R>(&self, f: F)
    where
        F: FnMut(Option<R>) -> R + 'static,
        R: 'static,
    {
        let id = EffectId::new();
        let effect = Effect { f: Box::new(f) };
        let subscriber = effect.into_any_subscriber();
        self.subscribers.borrow_mut().insert(id, subscriber);
        self.run_effect(&id);
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
