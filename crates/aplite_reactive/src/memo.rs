use std::sync::{Arc, Weak, RwLock};
use crate::graph::{ReactiveStorage, Node, Observer};
use crate::source::{AnySource, Source, ToAnySource};
use crate::subscriber::{AnySubscriber, Subscriber, SubscriberSet, ToAnySubscriber};
use crate::reactive_traits::*;
use crate::stored_value::Value;

/*
#########################################################
#
# MemoNode
#
#########################################################
*/

struct MemoState<T> {
    stored_value: Value<Option<T>>,
    f: Box<dyn Fn(Option<T>) -> (T, bool)>,
    state: RwLock<State>
}

struct State {
    sources: Vec<AnySource>,
    subscribers: SubscriberSet,
    this: AnySubscriber,
    dirty: bool,
}

unsafe impl<T> Send for MemoState<T> {}
unsafe impl<T> Sync for MemoState<T> {}

impl<T> MemoState<T> {
    fn new(f: Box<dyn Fn(Option<T>) -> (T, bool)>, this: AnySubscriber) -> Self {
        Self {
            stored_value: Value::new(None::<T>),
            f,
            state: RwLock::new(State {
                sources: Vec::new(),
                subscribers: SubscriberSet::default(),
                this,
                dirty: true,
            })
        }
    }

    #[inline(always)]
    fn read_value(&self) -> std::sync::RwLockReadGuard<'_, Option<T>> {
        self.stored_value.read().unwrap()
    }

    #[inline(always)]
    fn state_reader(&self) -> std::sync::RwLockReadGuard<'_, State> {
        self.state.read().unwrap()
    }

    #[inline(always)]
    fn state_writer(&self) -> std::sync::RwLockWriteGuard<'_, State> {
        self.state.write().unwrap()
    }
}

impl<T> Subscriber for MemoState<T> {
    fn add_source(&self, source: AnySource) {
        self.state_writer()
            .sources
            .push(source);
    }

    fn clear_sources(&self) {
        self.state_writer()
            .sources
            .clear();
    }
}

impl<T> Source for MemoState<T> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        self.state_writer()
            .subscribers
            .push(subscriber);
    }

    fn clear_subscribers(&self) {
        self.state_writer()
            .subscribers
            .clear();
    }
}

impl<T> Reactive for MemoState<T> {
    fn mark_dirty(&self) {
        self.state_writer().dirty = true;

        for sub in &self.state_reader().subscribers.0 {
            sub.mark_dirty();
        }
    }

    fn try_update(&self) -> bool {
        let state_read_lock = self.state_reader();

        if state_read_lock.dirty {
            let mut value_lock = self.stored_value.write().unwrap();
            let prev_value = value_lock.take();

            let this = state_read_lock.this.clone();
            drop(state_read_lock);
            this.clear_sources();

            let (new_value, changed) = this.as_observer(|| (self.f)(prev_value));
            *value_lock = Some(new_value);
            drop(value_lock);

            let mut state_write_lock = self.state_writer();
            state_write_lock.dirty = false;

            if changed {
                let subscribers = &state_write_lock.subscribers.0;

                Observer::with(|current| for sub in subscribers {
                    if current.is_some_and(|any| any != sub) {
                        sub.mark_dirty();
                    }
                });
            }

            return changed
        }

        false
    }
}

/*
#########################################################
#
# Memo
#
#########################################################
*/

pub struct Memo<T> {
    node: Node<Arc<MemoState<T>>>
}

impl<T: PartialEq + 'static> Memo<T> {
    pub fn new(f: impl Fn(Option<&T>) -> T + 'static) -> Self {
        let state = Arc::new_cyclic(move |weak| {
            let this = AnySubscriber::from_weak(Weak::clone(weak));

            let memoize_fn = move |prev: Option<T>| {
                let new_value = f(prev.as_ref());
                let changed = prev.as_ref() != Some(&new_value);
                (new_value, changed)
            };

            MemoState::new(Box::new(memoize_fn), this)
        });

        Self { node: ReactiveStorage::insert(state) }
    }
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self { node: self.node }
    }
}

impl<T> Copy for Memo<T> {}

impl<T: 'static> Source for Memo<T> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        ReactiveStorage::with_downcast(&self.node, |state| {
            state.add_subscriber(subscriber);
        })
    }

    fn clear_subscribers(&self) {
        ReactiveStorage::with_downcast(&self.node, |state| {
            state.clear_subscribers();
        })
    }
}

impl<T: 'static> ToAnySource for Memo<T> {
    fn to_any_source(&self) -> AnySource {
        ReactiveStorage::with_downcast(&self.node, AnySource::new)
    }
}

impl<T: 'static> Subscriber for Memo<T> {
    fn add_source(&self, source: AnySource) {
        ReactiveStorage::with_downcast(&self.node, |state| {
            state.add_source(source);
        })
    }

    fn clear_sources(&self) {
        ReactiveStorage::with_downcast(&self.node, |state| {
            state.clear_sources();
        })
    }
}

impl<T: 'static> ToAnySubscriber for Memo<T> {
    fn to_any_subscriber(&self) -> AnySubscriber {
        ReactiveStorage::with_downcast(&self.node, AnySubscriber::new)
    }
}

impl<T: 'static> Reactive for Memo<T> {
    fn mark_dirty(&self) {
        ReactiveStorage::with_downcast(&self.node, |state| {
            state.mark_dirty();
        })
    }

    fn try_update(&self) -> bool {
        ReactiveStorage::with_downcast(&self.node, |state| {
            state.try_update()
        })
    }
}

impl<T: 'static> Track for Memo<T> {
    fn track(&self) {
        Observer::with(|current| if let Some(any_subscriber) = current {
            any_subscriber.add_source(self.to_any_source());
            self.add_subscriber(any_subscriber.clone());
        })
    }

    fn untrack(&self) {
        self.clear_subscribers();
    }
}

impl<T: 'static> Read for Memo<T> {
    type Value = T;

    fn read<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R {
        ReactiveStorage::with_downcast(&self.node, |state| {
            state.try_update();
            f(state.read_value().as_ref().unwrap())
        })
    }

    fn try_read<R, F: FnOnce(Option<&Self::Value>) -> Option<R>>(&self, f: F) -> Option<R> {
        ReactiveStorage::try_with_downcast(&self.node, |opt| {
            opt.and_then(|state| {
                state.try_update();
                f(state.read_value().as_ref())
            })
        })
    }
}

impl<T: Clone + 'static> Get for Memo<T> {
    type Value = T;

    fn get_untracked(&self) -> Self::Value {
        self.read(Clone::clone)
    }

    fn try_get_untracked(&self) -> Option<Self::Value> {
        self.try_read(|value| value.cloned())
    }
}

impl<T: 'static> With for Memo<T> {
    type Value = T;

    fn with_untracked<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Value) -> R {
        self.read(f)
    }

    fn try_with_untracked<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(Option<&Self::Value>) -> Option<R> {
        self.try_read(f)
    }
}

#[cfg(test)]
mod memo_test {
    use aplite_future::{Executor, sleep};
    use crate::signal::Signal;
    use crate::effect::Effect;
    use super::*;

    #[test]
    fn effect() {
        Executor::init(3);

        let (first, set_first) = Signal::split("Dario");
        let (last, set_last) = Signal::split("");

        let full_name = Memo::new(move |_| {
            first.get().to_string() + last.get()
        });

        Effect::new(move |_| {
            full_name.with(|name| eprintln!("full name: {name}"))
        });

        let delta = 100;
        Executor::spawn(async move {
            let duration = std::time::Duration::from_millis(delta);

            set_first.set("Mario");
            sleep(duration).await;
            assert!(full_name.with(|name| name == "Mario"));

            set_last.set(" Ballotelli");
            sleep(duration).await;
            assert!(full_name.with(|name| name == "Mario Ballotelli"));

            set_last.set(" Nunez");
            sleep(duration).await;
            assert!(full_name.with(|name| name == "Mario Nunez"));

            set_first.set("Darwin");
            sleep(duration).await;
            assert!(full_name.with(|name| name == "Darwin Nunez"));
        });

        std::thread::sleep(std::time::Duration::from_millis(delta * 4));
    }
}
