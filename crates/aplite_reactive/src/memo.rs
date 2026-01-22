use std::sync::{Arc, Weak, RwLock};
use std::cell::UnsafeCell;

use crate::graph::{ReactiveStorage, Node, Observer, Scope};
use crate::source::{AnySource, Source, Sources, ToAnySource};
use crate::subscriber::{AnySubscriber, Subscriber, Subscribers, ToAnySubscriber};
use crate::reactive_traits::*;

pub struct Memo<T> {
    node: Node<Arc<MemoState<T>>>
}

struct MemoState<T> {
    scope: Scope,
    value: UnsafeCell<Option<T>>,
    f: Box<dyn Fn(Option<T>) -> (T, bool)>,
    state: RwLock<ReactiveState>
}

struct ReactiveState {
    sources: Sources,
    subscribers: Subscribers,
    this: AnySubscriber,
    dirty: bool,
}

unsafe impl<T> Send for MemoState<T> {}
unsafe impl<T> Sync for MemoState<T> {}

/*
#########################################################
#
# impl MemoState
#
#########################################################
*/

impl<T> MemoState<T> {
    fn new<F>(f: F, this: AnySubscriber) -> Self
    where
        F: Fn(Option<T>) -> (T, bool) + 'static,
    {
        Self {
            scope: Scope::new(),
            value: UnsafeCell::new(None::<T>),
            f: Box::new(f),
            state: RwLock::new(ReactiveState {
                sources: Sources::default(),
                subscribers: Subscribers::default(),
                this,
                dirty: true,
            })
        }
    }

    fn read_value(&self) -> &Option<T> {
        unsafe {
            &*self.value.get()
        }
    }

    fn write_value(&self) -> &mut Option<T> {
        unsafe {
            &mut *self.value.get()
        }
    }

    #[inline(always)]
    fn state_reader(&self) -> std::sync::RwLockReadGuard<'_, ReactiveState> {
        self.state.read().unwrap()
    }

    #[inline(always)]
    fn state_writer(&self) -> std::sync::RwLockWriteGuard<'_, ReactiveState> {
        self.state.write().unwrap()
    }
}

impl<T> Subscriber for MemoState<T> {
    fn add_source(&self, source: AnySource) {
        self.state_writer()
            .sources
            .add_source(source);
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
            let val = self.write_value();
            let prev_value = val.take();

            let this = state_read_lock.this.clone();
            drop(state_read_lock);
            this.clear_sources();

            let (new_value, changed) = self.scope.with_cleanup(|| {
                this.as_observer(|| (self.f)(prev_value))
            });

            val.replace(new_value);

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
# impl Memo
#
#########################################################
*/

impl<T: PartialEq + 'static> Memo<T> {
    pub fn new(f: impl Fn(Option<&T>) -> T + 'static) -> Self {
        Self::with_compare(f, |prev, new| prev != new)
    }
}

impl<T: 'static> Memo<T> {
    pub fn with_compare(
        memoize_fn: impl Fn(Option<&T>) -> T + 'static,
        compare_fn: impl Fn(Option<&T>, Option<&T>) -> bool + 'static,
    ) -> Self
    {
        let state = Arc::new_cyclic(move |weak| {
            let this = AnySubscriber::from_weak(Weak::clone(weak));

            let f = move |prev: Option<T>| {
                let new_value = memoize_fn(prev.as_ref());
                let changed = compare_fn(prev.as_ref(), Some(&new_value));
                (new_value, changed)
            };

            MemoState::new(f, this)
        });

        Self { node: ReactiveStorage::insert(state) }
    }
}

impl<T: 'static> Dispose for Memo<T> {
    fn dispose(&self) {
        let scope = ReactiveStorage::map_with_downcast(&self.node, |state| {
            state.scope.downgrade()
        });

        if let Some(scope) = scope.as_ref()
            .and_then(crate::graph::WeakScope::upgrade) {
            scope.cleanup();
        }

        ReactiveStorage::remove(self.node);
    }

    fn is_disposed(&self) -> bool {
        ReactiveStorage::is_removed(&self.node)
    }
}

impl<T: 'static> Source for Memo<T> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        ReactiveStorage::map_with_downcast(&self.node, |state| {
            state.add_subscriber(subscriber);
        });
    }

    fn clear_subscribers(&self) {
        ReactiveStorage::map_with_downcast(&self.node, |state| {
            state.clear_subscribers();
        });
    }
}

impl<T: 'static> ToAnySource for Memo<T> {
    fn to_any_source(&self) -> AnySource {
        ReactiveStorage::with_downcast(&self.node, AnySource::new)
    }
}

impl<T: 'static> Subscriber for Memo<T> {
    fn add_source(&self, source: AnySource) {
        ReactiveStorage::map_with_downcast(&self.node, |state| {
            state.add_source(source);
        });
    }

    fn clear_sources(&self) {
        ReactiveStorage::map_with_downcast(&self.node, |state| {
            state.clear_sources();
        });
    }
}

impl<T: 'static> ToAnySubscriber for Memo<T> {
    fn to_any_subscriber(&self) -> AnySubscriber {
        ReactiveStorage::with_downcast(&self.node, AnySubscriber::new)
    }
}

impl<T: 'static> Reactive for Memo<T> {
    fn mark_dirty(&self) {
        ReactiveStorage::map_with_downcast(&self.node, |state| {
            state.mark_dirty();
        });
    }

    fn try_update(&self) -> bool {
        ReactiveStorage::map_with_downcast(&self.node, Arc::downgrade)
            .and_then(|weak| weak.upgrade())
            .map(|arc| arc.try_update())
            .unwrap_or_default()
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
        self.try_update();

        ReactiveStorage::with_downcast(&self.node, |state| {
            f(state.read_value().as_ref().unwrap())
        })
    }

    fn try_read<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> Option<R> {
        self.try_update();

        ReactiveStorage::try_with_downcast(&self.node, |state| {
            state.read_value().as_ref().map(f)
        })
    }
}

impl<T: Clone + 'static> Get for Memo<T> {
    type Value = T;

    fn get_untracked(&self) -> Self::Value {
        self.read(Clone::clone)
    }

    fn try_get_untracked(&self) -> Option<Self::Value> {
        self.try_read(Clone::clone)
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
        F: FnOnce(&Self::Value) -> R {
        self.try_read(f)
    }
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self { node: self.node }
    }
}

impl<T> Copy for Memo<T> {}

impl<T: PartialEq + 'static> PartialEq for Memo<T> {
    fn eq(&self, o: &Self) -> bool {
        self.with_untracked(|this| {
            o.with_untracked(|other| {
                this == other
            })
        })
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
        let (name, set_name) = Signal::split("");

        let memoized = Memo::new(move |_| name.get());

        Effect::new(move |_| {
            memoized.with(|name| eprintln!("full name: {name}"))
        });

        let delta = 200;

        Executor::spawn(async move {
            let duration = std::time::Duration::from_millis(delta);

            set_name.set("Mario");
            sleep(duration).await;
            assert!(memoized.with_untracked(|&name| name == "Mario"));

            set_name.set("Ballotelli");
            sleep(duration).await;
            assert!(memoized.with_untracked(|&name| name == "Ballotelli"));

            set_name.set("Darwin");
            sleep(duration).await;
            assert!(memoized.with_untracked(|&name| name == "Darwin"));

            set_name.set("Nunez");
            sleep(duration).await;
            assert!(memoized.with_untracked(|&name| name == "Nunez"));
        });

        std::thread::sleep(std::time::Duration::from_millis(delta * 4));
    }

    #[test]
    fn parent_child() {
        let name = Signal::new("Signal");

        let parent = Memo::new(move |_| {
            let child = Memo::new(move |_| name.get());

            Effect::new(move |_| {
                child.with(|name| eprintln!("signal: {name}"))
            });

            child
        });

        let child = parent.get();
        std::thread::sleep(std::time::Duration::from_millis(200));

        let is_disposed = child.is_disposed();
        assert!(!is_disposed);
        println!("child is disposed: {}", is_disposed);

        parent.dispose();

        let is_disposed = child.is_disposed();
        assert!(is_disposed);
        println!("child is disposed: {}", is_disposed);
    }
}
