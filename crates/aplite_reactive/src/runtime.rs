use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::marker::PhantomData;

use aplite_storage::{Key, VecMap};

use crate::effect::Effect;
use crate::signal::Signal;
use crate::RwSignal;

thread_local! {
    pub static RUNTIME: ReactiveGraph = ReactiveGraph::new();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReactiveId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct EffectId;

type Subscribers = HashMap<Key<ReactiveId>, HashSet<Key<EffectId>>>;

pub(crate) struct ReactiveGraph {
    pub(crate) storage: RefCell<VecMap<ReactiveId, Signal>>,
    pub(crate) current: RefCell<Option<Key<EffectId>>>,
    pub(crate) subscribers: RefCell<Subscribers>,
    pub(crate) effects: RefCell<VecMap<EffectId, Effect>>,
}

impl ReactiveGraph {
    pub(crate) fn new() -> Self {
        Self {
            storage: RefCell::new(VecMap::new()),
            current: RefCell::new(None),
            subscribers: RefCell::new(HashMap::new()),
            effects: RefCell::new(VecMap::new()),
        }
    }

    pub(crate) fn create_rw_signal<T: Any + 'static>(&self, value: T) -> RwSignal<T> {
        let id = self.storage.borrow_mut().insert(Signal::stored(value));
        RwSignal { id, phantom: PhantomData }
    }

    pub(crate) fn create_effect<F: Fn() + 'static>(&self, f: F) {
        let effect = Effect { f: Box::new(f) };
        let id = self.effects.borrow_mut().insert(effect);
        self.run_effect(&id);
    }

    fn run_effect(&self, effect_id: &Key<EffectId>) {
        let pref_effect = self.current.take();
        *self.current.borrow_mut() = Some(*effect_id);
        if let Some(effect) = self.effects.borrow().get(effect_id) {
            effect.run();
        }
        *self.current.borrow_mut() = pref_effect;
    }

    pub(crate) fn add_subscriber(&self, id: Key<ReactiveId>) {
        let current = self.current.borrow();
        if let Some(effect_id) = current.as_ref() {
            let mut subscribers = self.subscribers.borrow_mut();
            subscribers.entry(id).or_default().insert(*effect_id);
        }
    }

    pub(crate) fn notify_subscribers(&self, id: Key<ReactiveId>) {
        let subsribers = self.subscribers.borrow();
        let maybe_subs = subsribers.get(&id).cloned();
        drop(subsribers);
        if let Some(subscribers) = maybe_subs {
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
