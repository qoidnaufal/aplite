use std::sync::{Arc, RwLock};
use aplite_future::{
    Channel,
    Sender,
    Receiver,
    Executor,
};

use crate::graph::{ReactiveNode, GRAPH};
use crate::subscriber::{Subscriber, ToAnySubscriber, AnySubscriber};
use crate::source::AnySource;
use crate::reactive_traits::*;

/// [`Effect`] is a scope to synchronize the reactive node (eg: [`Signal`](crate::signal::Signal)) with anything.
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
    node: ReactiveNode<Arc<Scope>>,
}

impl Effect {
    pub fn new<F, R>(f: F) -> Self
    where
        F: FnMut(Option<R>) -> R + 'static,
        R: 'static,
    {
        let (tx, rx) = Channel::new();
        tx.notify();
        Self::with_scope(Scope::new(tx), rx, f)
    }

    pub fn with_scope<F, R>(scope: Scope, mut rx: Receiver, mut f: F) -> Self
    where
        F: FnMut(Option<R>) -> R + 'static,
        R: 'static,
    {
        let scope = Arc::new(scope);
        let node = GRAPH.with(|graph| {
            graph.insert(Arc::clone(&scope))
        });
        let scope = scope.to_any_subscriber();
        let this = Self { node };

        Executor::spawn_local(async move {
            let value = Arc::new(RwLock::new(None::<R>));

            while rx.recv().await.is_some() {
                #[cfg(test)] eprintln!("\n[NOTIFIED]      : {:?}", this);

                let prev_scope = GRAPH.with(|graph| graph.set_scope(Some(scope.clone())));

                scope.clear_source();

                let mut lock = value.write().unwrap();
                let prev_value = lock.take();
                let new_val = f(prev_value);

                *lock = Some(new_val);

                GRAPH.with(|graph| graph.set_scope(prev_scope));

                let source_count = scope.source_count();
                #[cfg(test)] eprintln!("current source count: {source_count}");
                if source_count == 0 { break; }
            }

            drop(rx);
            drop(scope);
            GRAPH.with(|graph| graph.remove(&node));
        });

        this
    }
}

pub struct Scope {
    pub(crate) sender: Sender,
    pub(crate) source: RwLock<Vec<AnySource>>,
}

impl Scope {
    pub fn new(sender: Sender) -> Self {
        Self {
            sender,
            source: RwLock::new(Vec::new()),
        }
    }
}

impl Subscriber for Scope {
    fn add_source(&self, source: AnySource) {
        self.source.write().unwrap().push(source);
    }

    fn clear_source(&self) {
        let mut sources = self.source.write().unwrap();
        let drained_sources = sources.drain(..);
        drained_sources.into_iter().for_each(|source| source.untrack())
    }

    fn source_count(&self) -> usize {
        self.source.try_read().unwrap().len()
    }
}

impl Subscriber for Arc<Scope> {
    fn add_source(&self, source: AnySource) {
        self.as_ref().add_source(source);
    }

    fn clear_source(&self) {
        self.as_ref().clear_source();
    }

    fn source_count(&self) -> usize {
        self.as_ref().source_count()
    }
}

impl Notify for Scope {
    fn notify(&self) {
        self.sender.notify();
    }
}

impl Notify for Arc<Scope> {
    fn notify(&self) {
        self.as_ref().notify();
    }
}

impl ToAnySubscriber for Arc<Scope> {
    fn to_any_subscriber(self) -> AnySubscriber {
        AnySubscriber::new(self)
    }
}

impl Track for Scope {
    fn track(&self) {}
    fn untrack(&self) {}
}

impl Track for Arc<Scope> {
    fn track(&self) {}
    fn untrack(&self) {}
}

impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scope")
            .field("source_count", &self.source.read().map(|s| s.len()).unwrap_or_default())
            .finish()
    }
}

#[cfg(test)]
mod effect_test {
    use std::rc::Rc;
    use std::cell::RefCell;
    use aplite_future::{Runtime, Executor, sleep};
    use crate::signal::Signal;
    use crate::reactive_traits::*;
    use super::*;

    #[test]
    fn effect() {
        let rt = Runtime::init();

        let use_last = Signal::new(false);
        let (first, set_first) = Signal::split("Dario");
        let (last, set_last) = Signal::split("");

        let name = Rc::new(RefCell::new(String::new()));
        let set_name = Rc::clone(&name);

        rt.spawn_local(async move {
            Effect::new(move |_| {
                if use_last.get() {
                    *set_name.borrow_mut() = first.get().to_string() + " " + last.get();
                } else {
                    *set_name.borrow_mut() = first.with(|n| n.to_string());
                }
            });

            Effect::new(move |_| eprintln!("last name: {}", last.get_untracked()));
        });

        Executor::spawn_local(async move {
            sleep(1000).await;
            set_first.set("Mario");

            set_last.set("Ballotelli");
            sleep(1000).await;
            assert_eq!("Mario", name.borrow().as_str());

            use_last.set(true);
            sleep(1000).await;
            assert_eq!("Mario Ballotelli", name.borrow().as_str());

            use_last.set(false);
            sleep(1000).await;
            assert_eq!("Mario", name.borrow().as_str());

            set_last.set("Gomez");
            sleep(1000).await;
            assert_eq!("Mario", name.borrow().as_str());

            set_last.set("Bros");
            sleep(1000).await;
            assert_eq!("Mario", name.borrow().as_str());

            set_last.set("Kempes");
            sleep(1000).await;
            assert_eq!("Mario", name.borrow().as_str());

            use_last.set(true);
            sleep(1000).await;
            assert_eq!("Mario Kempes", name.borrow().as_str());
        });

        rt.run();
    }
}
