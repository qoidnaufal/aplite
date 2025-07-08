use std::any::Any;
use std::marker::PhantomData;
use std::{cell::RefCell, collections::HashMap};

use crate::signal::Signal;
use crate::graph::{EffectId, ReactiveId, GRAPH};
use crate::subscriber::{AnySubscriber, ToAnySubscriber};
use crate::effect::Effect;
use crate::{EffectInner, RwSignal};

pub struct ReactiveScope {
    owner: Effect,
    signals: RefCell<HashMap<ReactiveId, Signal>>,
    subscribers: RefCell<HashMap<EffectId, AnySubscriber>>,
}

impl ReactiveScope {
    pub fn new() -> Self {
        let owner = Effect::new(|_| {});
        Self {
            owner,
            signals: RefCell::new(HashMap::new()),
            subscribers: RefCell::new(HashMap::new()),
        }
    }

    pub fn create_rw_signal<T: Any + 'static>(&self, value: T) -> RwSignal<T> {
        let id = ReactiveId::new();
        let signal = Signal::store_value(value);
        signal.add_subscriber(self.owner);
        self.signals.borrow_mut().insert(id, signal);
        RwSignal { id, phantom: PhantomData }
    }

    pub fn create_effect<F, R>(&self, f: F) -> Effect
    where
        F: FnMut(Option<R>) -> R + 'static,
        R: 'static,
    {
        let id = EffectId::new();
        let subscriber = RefCell::new(EffectInner::new(f));
        self.subscribers
            .borrow_mut()
            .insert(id, subscriber.to_any_subscriber());
        let effect = Effect::with_id(id);
        effect
    }
}
