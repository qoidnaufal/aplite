use std::sync::{Arc, RwLock};
use aplite_future::{
    aplite_channel,
    Sender,
    Executor,
};

use crate::graph::{Node, ReactiveStorage, Scope};
use crate::subscriber::{Subscriber, ToAnySubscriber, AnySubscriber};
use crate::source::{AnySource, Sources};
use crate::reactive_traits::*;

/// [`Effect`] is an async scope to synchronize reactive node (eg: [`Signal`](crate::signal::Signal)) with anything.
/// # Example
/// ```ignore
/// let (counter, set_counter) = Signal::split(0i32);
/// Effect::new(move |_| println!("{}", counter.get()));
///
/// // and then do something with the set_counter
/// let on_click = move || set_counter.update(|num| *num += 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Effect {
    node: EffectNode,
}

type EffectNode = Node<Arc<RwLock<EffectState>>>;

impl Effect {
    pub fn pause(&self) { }
}

impl Effect {
    pub fn new<F, R>(mut f: F) -> Self
    where
        F: FnMut(Option<R>) -> R + 'static,
        R: 'static,
    {
        let (tx, mut rx) = aplite_channel();
        tx.notify();

        let scope = Scope::new();
        let effect_state = EffectState::new(tx);
        let subscriber = effect_state.to_any_subscriber();
        let node = ReactiveStorage::insert(effect_state);

        Executor::spawn(async move {
            let mut value = None::<R>;

            while rx.recv().await.is_some() {
                if !scope.is_paused() && subscriber.needs_update() {
                    subscriber.clear_sources();

                    let prev_value = value.take();
                    let new_value = scope.with_cleanup(|| {
                        subscriber.as_observer(|| {
                            f(prev_value)
                        })
                    });

                    value = Some(new_value);
                }
            }
        });

        Self { node }
    }

    /// For now a simple brute-force by removing the EffectState from NodeStorage is enough.
    /// Telling the sources to remove this node is unnecessary, as the next [`Reactive::mark_dirty`] by each sources will clean-up their subscribers.
    /// Since this node has been removed from the main storage, upgrading the [`Weak`](std::sync::Weak) references will result in [`None`] and nothing happens.
    pub fn stop(self) {
        ReactiveStorage::remove(self.node);
    }
}

struct EffectState {
    sender: Sender,
    source: Sources,
    dirty: bool,
}

unsafe impl Send for EffectState {}
unsafe impl Sync for EffectState {}

impl EffectState {
    fn new(sender: Sender) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            sender,
            source: Sources::default(),
            dirty: true,
        }))
    }
}

impl Drop for EffectState {
    fn drop(&mut self) {
        self.source.clear();
    }
}

// ---- impl Subscriber

impl Subscriber for RwLock<EffectState> {
    fn add_source(&self, source: AnySource) {
        let mut lock = self.write().unwrap();
        lock.source.add_source(source);
    }

    fn clear_sources(&self) {
        self.write().unwrap().source.clear();
    }
}

impl Subscriber for Arc<RwLock<EffectState>> {
    fn add_source(&self, source: AnySource) {
        self.as_ref().add_source(source);
    }

    fn clear_sources(&self) {
        self.as_ref().clear_sources();
    }
}

// ---- impl Reactive

impl Reactive for RwLock<EffectState> {
    fn mark_dirty(&self) {
        let this = &mut *self.write().unwrap();
        this.dirty = true;
        this.sender.notify();
    }

    fn try_update(&self) -> bool {
        let mut lock = self.write().unwrap();
        if lock.dirty {
            lock.dirty = false;
            return true;
        }

        lock.source.try_update()
    }
}

impl Reactive for Arc<RwLock<EffectState>> {
    fn mark_dirty(&self) {
        self.as_ref().mark_dirty();
    }

    fn try_update(&self) -> bool {
        self.as_ref().try_update()
    }
}

// ---- impl ToAnySubscriber

impl ToAnySubscriber for Arc<RwLock<EffectState>> {
    fn to_any_subscriber(&self) -> AnySubscriber {
        AnySubscriber::new(self)
    }
}

/*
#########################################################
#
# Test
#
#########################################################
*/

#[cfg(test)]
mod effect_test {
    use std::rc::Rc;
    use std::cell::RefCell;
    use aplite_future::{Executor, sleep};
    use crate::signal::Signal;
    use crate::reactive_traits::*;
    use super::*;

    #[test]
    fn effect() {
        let use_last = Signal::new(false);
        let (first, set_first) = Signal::split("Dario");
        let (last, set_last) = Signal::split("");

        let name = Rc::new(RefCell::new(String::new()));
        let cloned_rc = Rc::clone(&name);

        Effect::new(move |_| {
            if use_last.get() {
                *cloned_rc.borrow_mut() = first.get().to_string() + " " + last.get();
            } else {
                *cloned_rc.borrow_mut() = first.with(|n| n.to_string());
            }

            eprintln!("full name: {}", *cloned_rc.borrow());
        });

        let print_effect = Effect::new(move |_| eprintln!("last name: {}", last.get()));

        let delta = 100;
        Executor::spawn(async move {
            let duration = std::time::Duration::from_millis(delta);

            set_first.set("Mario");
            sleep(duration).await;

            set_last.set("Ballotelli");
            sleep(duration).await;
            assert_eq!("Mario", name.borrow().as_str());

            print_effect.stop();

            use_last.set(true);
            sleep(duration).await;
            assert_eq!("Mario Ballotelli", name.borrow().as_str());

            use_last.set(false);
            sleep(duration).await;
            assert_eq!("Mario", name.borrow().as_str());

            set_last.set("Gomez");
            sleep(duration).await;
            assert_eq!("Mario", name.borrow().as_str());

            set_last.set("Bros");
            sleep(duration).await;
            assert_eq!("Mario", name.borrow().as_str());

            set_last.set("Kempes");
            sleep(duration).await;
            assert_eq!("Mario", name.borrow().as_str());

            use_last.set(true);
            sleep(duration).await;
            assert_eq!("Mario Kempes", name.borrow().as_str());
        });

        std::thread::sleep(std::time::Duration::from_millis(delta * 9));
    }
}
