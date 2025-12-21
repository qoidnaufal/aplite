use std::sync::{Arc, RwLock};
use aplite_future::{
    aplite_channel,
    Sender,
    Executor,
};

use crate::graph::{Scope, Node, Graph};
use crate::subscriber::{Subscriber, ToAnySubscriber, AnySubscriber};
use crate::source::AnySource;
use crate::reactive_traits::*;

/// [`Effect`] is an async scope to synchronize reactive node (eg: [`Signal`](crate::signal::Signal)) with anything.
/// # Example
/// ```ignore
/// let (counter, set_counter) = Signal::split(0i32);
/// Effect::new(move |_| eprintln!("{}", counter.get()));
///
/// // and then do something with the set_counter
/// let on_click = move || set_counter.update(|num| *num += 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Effect {
    node: Node<Arc<RwLock<EffectNode>>>,
}

impl Effect {
    pub fn pause(&self) { }
}

impl Effect {
    pub fn new<F, R>(mut f: F) -> Self
    where
        F: FnMut(Option<R>) -> R + 'static,
        R: std::fmt::Debug + 'static,
    {
        let (tx, mut rx) = aplite_channel();
        tx.notify();

        let scope = Scope::new();
        let node = Arc::new(RwLock::new(EffectNode::new(tx)));
        let subscriber = node.to_any_subscriber();
        let node = Graph::insert(node);

        Executor::spawn(async move {
            let mut value = None::<R>;

            while rx.recv().await.is_some() {
                if !scope.paused() && subscriber.try_update() {
                    subscriber.clear_sources();

                    scope.with_cleanup(|| subscriber.as_observer(|| {
                        let prev_value = value.take();
                        let new_val = f(prev_value);
                        value = Some(new_val);
                    }))
                }
            }

            Graph::with_mut(|graph| graph.storage.remove(node.id));
        });

        Self { node }
    }
}

pub struct EffectNode {
    pub(crate) sender: Sender,
    pub(crate) source: Vec<AnySource>,
    pub(crate) dirty: bool,
}

unsafe impl Send for EffectNode {}
unsafe impl Sync for EffectNode {}

impl EffectNode {
    pub fn new(sender: Sender) -> Self {
        Self {
            sender,
            source: Vec::new(),
            dirty: true,
        }
    }
}

impl Drop for EffectNode {
    fn drop(&mut self) {
        self.source.clear();
    }
}

// ----

impl Subscriber for RwLock<EffectNode> {
    fn add_source(&self, source: AnySource) {
        let mut lock = self.write().unwrap();
        if !lock.source.contains(&source) { lock.source.push(source) }
    }

    fn clear_sources(&self) {
        self.write().unwrap().source.clear();
    }
}

impl Subscriber for Arc<RwLock<EffectNode>> {
    fn add_source(&self, source: AnySource) {
        self.as_ref().add_source(source);
    }

    fn clear_sources(&self) {
        self.as_ref().clear_sources();
    }
}

// ----

impl Notify for RwLock<EffectNode> {
    fn notify(&self) {
        let lock = &mut *self.write().unwrap();
        lock.dirty = true;
        lock.sender.notify();
    }
}

impl Notify for Arc<RwLock<EffectNode>> {
    fn notify(&self) {
        self.as_ref().notify();
    }
}

// ----

impl Reactive for RwLock<EffectNode> {
    fn update_if_necessary(&self) -> bool {
        let mut lock = self.write().unwrap();
        if lock.dirty {
            lock.dirty = false;
            return true;
        }

        let sources = lock.source.clone();
        drop(lock);

        sources.iter().any(AnySource::update_if_necessary)
    }
}

impl Reactive for Arc<RwLock<EffectNode>> {
    fn update_if_necessary(&self) -> bool {
        self.as_ref().update_if_necessary()
    }
}

// ----

impl ToAnySubscriber for Arc<RwLock<EffectNode>> {
    fn to_any_subscriber(&self) -> AnySubscriber {
        AnySubscriber::new(Arc::downgrade(self))
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
        Executor::init(3);

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

        Effect::new(move |_| eprintln!("last name: {}", last.get()));

        let delta = 100;
        Executor::spawn(async move {
            let duration = std::time::Duration::from_millis(delta);

            set_first.set("Mario");
            sleep(duration).await;

            set_last.set("Ballotelli");
            sleep(duration).await;
            assert_eq!("Mario", name.borrow().as_str());

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
