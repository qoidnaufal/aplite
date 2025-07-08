use std::any::Any;
use std::collections::HashMap;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::atomic::AtomicU64;

use crate::effect::{Effect, EffectInner};
use crate::signal::Signal;
use crate::subscriber::{AnySubscriber, ToAnySubscriber};
use crate::rw_signal::RwSignal;

thread_local! {
    pub(crate) static GRAPH: ReactiveGraph = ReactiveGraph::new();
}

/*
#########################################################
#                                                       #
#                          Id                           #
#                                                       #
#########################################################
*/

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

/*
#########################################################
#                                                       #
#                         Graph                         #
#                                                       #
#########################################################
*/

type SignalStorage = HashMap<ReactiveId, Signal>;
type Subscribers = HashMap<EffectId, AnySubscriber>;

pub(crate) struct ReactiveGraph {
    pub(crate) signals: RefCell<SignalStorage>,
    pub(crate) current: RefCell<Option<Effect>>,
    pub(crate) subscribers: RefCell<Subscribers>,
}

impl ReactiveGraph {
    pub(crate) fn new() -> Self {
        Self {
            signals: Default::default(),
            current: Default::default(),
            subscribers: Default::default(),
        }
    }

    pub(crate) fn create_rw_signal<T: Any + 'static>(&self, value: T) -> RwSignal<T> {
        let id = ReactiveId::new();
        self.signals.borrow_mut().insert(id, Signal::store_value(value));
        RwSignal { id, phantom: PhantomData }
    }

    pub(crate) fn create_effect<F, R>(&self, f: F) -> Effect
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
        self.run_effect(effect);
        effect
    }

    pub(crate) fn track(&self, id: &ReactiveId) {
        if let Some(signal) = self.signals.borrow().get(id) {
            let current = *self.current.borrow();
            if let Some(effect) = current {
                signal.add_subscriber(effect);
            }
        }
    }

    pub(crate) fn untrack(&self, id: &ReactiveId) {
        if let Some(signal) = self.signals.borrow().get(id) {
            signal.clear_subscribers();
        }
    }

    pub(crate) fn notify_subscribers(&self, id: &ReactiveId) {
        if let Some(subscribers) = self.get_subscribers(id) {
            self.untrack(id);
            subscribers
                .iter()
                .for_each(|effect| {
                    self.run_effect(*effect);
                });
        }
    }

    fn get_subscribers(&self, id: &ReactiveId) -> Option<Vec<Effect>> {
        self.signals
            .borrow()
            .get(id)
            .map(|s| s.get_subscribers())
    }

    fn run_effect(&self, effect: Effect) {
        let pref_effect = self.current.borrow_mut().replace(effect);

        let subscribers = self.subscribers.borrow();
        let subscriber = subscribers.get(effect.id()).cloned();

        drop(subscribers);

        if let Some(subscriber) = subscriber {
            subscriber.run()
        }

        *self.current.borrow_mut() = pref_effect;
    }
}

#[cfg(test)]
mod reactive_test {
    use std::rc::Rc;
    use std::cell::RefCell;
    use crate::Signal;
    use crate::RwSignal;
    use crate::{reactive_traits::*, Effect};

    #[test]
    fn signal() {
        let (counter, set_counter) = Signal::new(0i32);

        set_counter.update(|num| *num += 1);
        assert_eq!(counter.get(), 1);

        set_counter.set(-69);
        assert_eq!(counter.get(), -69);

        let r = counter.with(|num| num.to_string());
        assert_eq!(r.parse(), Ok(-69));
    }

    #[test]
    fn effect() {
        let (use_last, set_use_last) = Signal::new(false);
        let (first, set_first) = Signal::new("Dario");
        let (last, set_last) = Signal::new("");

        let name = Rc::new(RefCell::new(String::new()));
        let set_name = Rc::clone(&name);

        Effect::new(move |_| {
            if use_last.get() {
                *set_name.borrow_mut() = first.get().to_string() + " " + last.get();
            } else {
                *set_name.borrow_mut() = first.with(|n| n.to_string());
            }
        });

        set_first.set("Mario");
        set_last.set("Ballotelli");
        assert_eq!("Mario", name.borrow().as_str());

        set_use_last.set(true);
        assert_eq!("Mario Ballotelli", name.borrow().as_str());

        set_use_last.set(false);
        assert_eq!("Mario", name.borrow().as_str());

        set_last.set("Gomez");
        assert_eq!("Mario", name.borrow().as_str());

        set_last.set("Bros");
        assert_eq!("Mario", name.borrow().as_str());

        set_last.set("Kempes");
        assert_eq!("Mario", name.borrow().as_str());

        set_use_last.set(true);
        assert_eq!("Mario Kempes", name.borrow().as_str());
    }

    #[test]
    fn derive() {
        let rw = RwSignal::new(0i32);
        let (counter, set_counter) = Signal::new(0i32);

        set_counter.set(69);
        rw.update(|num| *num = counter.get());
        assert_eq!(rw.get(), 69);
    }

    #[test]
    fn child_effect() {
        let (check, set_check) = Signal::new(false);
        let (outer_name, set_outer_name) = Signal::new("Steve");

        let someone = Rc::new(RefCell::new(String::new()));
        let outer_one = Rc::clone(&someone);

        Effect::new(move |_| {
            let (inner_name, set_inner_name) = Signal::new("");
            let inner_one = Rc::clone(&outer_one);

            Effect::new(move |_| {
                if check.get() {
                    inner_name.with(|n| *inner_one.borrow_mut() = n.to_string());
                }
            });

            if check.get() {
                set_inner_name.set("Oscar");
            } else {
                *outer_one.borrow_mut() = outer_name.get().to_string();
            }
        });

        assert_eq!(someone.borrow().as_str(), "Steve");

        set_check.set(true);
        assert_eq!(someone.borrow().as_str(), "Oscar");

        set_outer_name.set("Douglas");

        set_check.set(false);
        assert_eq!(someone.borrow().as_str(), "Douglas");

        set_check.set(true);
        assert_eq!(someone.borrow().as_str(), "Oscar");
    }

    #[test]
    #[should_panic]
    fn dispose() {
        let (num, set_num) = Signal::new(0i32);
        let double = || num.get() * 2;

        set_num.set(1);
        assert_eq!(double(), 2);

        num.dispose();
        set_num.set(2);
        assert_eq!(double(), 2);
    }
}
