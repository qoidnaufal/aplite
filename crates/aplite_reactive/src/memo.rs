use std::sync::{Arc, Weak, RwLock};
use crate::graph::{Graph, Node, Observer, Scope};
use crate::source::{AnySource, Source, ToAnySource};
use crate::subscriber::{AnySubscriber, Subscriber, SubscriberSet, ToAnySubscriber};
use crate::reactive_traits::*;

/*
#########################################################
#
# MemoNode
#
#########################################################
*/

pub(crate) struct MemoNode<T> {
    value: Arc<RwLock<Option<T>>>,
    f: Arc<dyn Fn(Option<T>) -> (T, bool)>,
    scope: Scope,
    state: RwLock<MemoState>
}

pub(crate) struct MemoState {
    sources: Vec<AnySource>,
    subscribers: SubscriberSet,
    this: AnySubscriber,
    dirty: bool,
}

unsafe impl<T> Send for MemoNode<T> {}
unsafe impl<T> Sync for MemoNode<T> {}

impl<T> MemoNode<T> {
    fn new(f: Arc<dyn Fn(Option<T>) -> (T, bool)>, this: AnySubscriber) -> Self {
        Self {
            value: Arc::new(RwLock::new(None)),
            f,
            scope: Scope::new(),
            state: RwLock::new(MemoState {
                sources: Vec::new(),
                subscribers: SubscriberSet::default(),
                this,
                dirty: true,
            })
        }
    }

    fn read_value(&self) -> std::sync::RwLockReadGuard<'_, Option<T>> {
        self.value.read().unwrap()
    }

    #[inline(always)]
    fn state_reader(&self) -> std::sync::RwLockReadGuard<'_, MemoState> {
        self.state.read().unwrap()
    }

    #[inline(always)]
    fn state_writer(&self) -> std::sync::RwLockWriteGuard<'_, MemoState> {
        self.state.write().unwrap()
    }
}

impl<T> Subscriber for MemoNode<T> {
    fn add_source(&self, source: AnySource) {
        self.state.write()
            .unwrap()
            .sources
            .push(source);
    }

    fn clear_sources(&self) {
        self.state.write()
            .unwrap()
            .sources
            .clear();
    }
}

impl<T> Source for MemoNode<T> {
    fn add_subscriber(&self, subscriber: AnySubscriber) {
        self.state.write()
            .unwrap()
            .subscribers.0
            .push(subscriber);
    }

    fn clear_subscribers(&self) {
        self.state.write()
            .unwrap()
            .subscribers.0
            .clear();
    }
}

impl<T> Reactive for MemoNode<T> {
    fn mark_dirty(&self) {
        self.state.write().unwrap().dirty = true;
        for sub in &self.state.read().unwrap().subscribers.0 {
            sub.mark_dirty();
        }
    }

    fn update_if_necessary(&self) -> bool {
        let state_read_lock = self.state_reader();

        if state_read_lock.dirty {
            let mut value_lock = self.value.write().unwrap();
            let value = value_lock.take();

            let this = state_read_lock.this.clone();
            drop(state_read_lock);
            this.clear_sources();

            let (new_value, changed) = self.scope.with_cleanup(|| {
                let prev = Observer::swap_observer(Some(this));
                let (new_value, changed) = (self.f)(value);
                Observer::swap_observer(prev);
                (new_value, changed)
            });

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

            drop(state_write_lock);

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
    node: Node<Arc<MemoNode<T>>>
}

impl<T: 'static> Memo<T> {
    pub fn new(f: impl Fn(Option<&T>) -> T + 'static) -> Self
    where
        T: PartialEq,
    {
        let arc_node = Arc::new_cyclic(move |weak| {
            let any_subscriber = AnySubscriber::new(Weak::clone(weak));

            let memoize_fn = move |prev: Option<T>| {
                let new_value = f(prev.as_ref());
                let changed = prev.as_ref() != Some(&new_value);
                (new_value, changed)
            };

            MemoNode::new(Arc::new(memoize_fn), any_subscriber)
        });

        Self { node: Graph::insert(arc_node) }
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
        Graph::with_downcast(&self.node, |memo_node| {
            memo_node.add_subscriber(subscriber);
        })
    }

    fn clear_subscribers(&self) {
        Graph::with_downcast(&self.node, |memo_node| {
            memo_node.clear_subscribers();
        })
    }
}

impl<T: 'static> ToAnySource for Memo<T> {
    fn to_any_source(&self) -> AnySource {
        AnySource::new(self.node.id)
    }
}

impl<T: 'static> Subscriber for Memo<T> {
    fn add_source(&self, source: AnySource) {
        Graph::with_downcast(&self.node, |memo_node| {
            memo_node.add_source(source);
        })
    }

    fn clear_sources(&self) {
        Graph::with_downcast(&self.node, |memo_node| {
            memo_node.clear_sources();
        })
    }
}

impl<T: 'static> ToAnySubscriber for Memo<T> {
    fn to_any_subscriber(&self) -> AnySubscriber {
        Graph::with_downcast(
            &self.node,
            |memo_node| AnySubscriber::new(Arc::downgrade(memo_node))
        )
    }
}

impl<T: 'static> Reactive for Memo<T> {
    fn mark_dirty(&self) {
        Graph::with_downcast(&self.node, |memo_node| {
            memo_node.mark_dirty();
        })
    }

    fn update_if_necessary(&self) -> bool {
        Graph::with_downcast(&self.node, |memo_node| {
            memo_node.update_if_necessary()
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
        let node = Graph::with_downcast(&self.node, Arc::clone);
        node.update_if_necessary();
        f(node.read_value().as_ref().unwrap())
    }

    fn try_read<R, F: FnOnce(Option<&Self::Value>) -> Option<R>>(&self, f: F) -> Option<R> {
        let node = Graph::try_with_downcast(&self.node, |opt| opt.map(Arc::clone));
        node.and_then(|node| {
            node.update_if_necessary();
            f(node.read_value().as_ref())
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
