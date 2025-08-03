use std::sync::{Arc, RwLock};
use aplite_future::{
    Channel,
    Sender,
    Executor,
};

use crate::graph::GRAPH;
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
    // node: ReactiveNode<Arc<Reactor>>,
}

impl Effect {
    pub fn new<F, R>(mut f: F) -> Self
    where
        F: FnMut(Option<R>) -> R + 'static,
        R: 'static,
    {
        let (tx, mut rx) = Channel::new();
        tx.notify();

        let reactor = Arc::new(Reactor::new(tx));
        // let node = GRAPH.with(|graph| {
        //     graph.insert(Arc::clone(&reactor))
        // });

        Executor::spawn_local(async move {
            let value = Arc::new(RwLock::new(None::<R>));

            while rx.recv().await.is_some() {
                #[cfg(test)] eprintln!("\n[NOTIFIED]      : {:?}", reactor);

                let prev_scope = GRAPH.with(|graph| {
                    let subscriber = reactor.clone().to_any_subscriber();
                    graph.swap_current(Some(subscriber))
                });
                reactor.clear_source();

                let mut lock = value.write().unwrap();
                let prev_value = lock.take();
                let new_val = f(prev_value);
                *lock = Some(new_val);

                GRAPH.with(|graph| graph.swap_current(prev_scope));
            }
        });

        Self { }
    }
}

pub(crate) struct Reactor {
    pub(crate) sender: Sender,
    pub(crate) source: RwLock<Vec<AnySource>>,
}

impl Reactor {
    fn new(sender: Sender) -> Self {
        Self {
            sender,
            source: RwLock::new(Vec::new()),
        }
    }
}

impl Subscriber for Reactor {
    fn add_source(&self, source: AnySource) {
        self.source.write().unwrap().push(source);
    }

    fn clear_source(&self) {
        let mut sources = self.source.write().unwrap();
        let drained_sources = sources.drain(..);
        drained_sources.into_iter().for_each(|source| source.untrack())
    }
}

impl Subscriber for Arc<Reactor> {
    fn add_source(&self, source: AnySource) {
        self.as_ref().add_source(source);
    }

    fn clear_source(&self) {
        self.as_ref().clear_source();
    }
}

impl Notify for Reactor {
    fn notify(&self) {
        self.sender.notify();
    }
}

impl Notify for Arc<Reactor> {
    fn notify(&self) {
        self.as_ref().notify();
    }
}

impl ToAnySubscriber for Arc<Reactor> {
    fn to_any_subscriber(self) -> AnySubscriber {
        AnySubscriber::new(self)
    }
}

impl Track for Reactor {
    fn track(&self) {}
    fn untrack(&self) {}
}

impl Track for Arc<Reactor> {
    fn track(&self) {}
    fn untrack(&self) {}
}

impl std::fmt::Debug for Reactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reactor")
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
                    *set_name.borrow_mut() = first.read(|n| n.to_string());
                }
            });

            Effect::new(move |_| eprintln!("last name: {}", last.get()));
        });

        // Executor::spawn_local(async {
        //     aplite_future::block_on(sleep(4000));
        //     eprintln!("done");
        // });

        Executor::spawn_local(async move {
            set_first.set("Mario");
            sleep(1000).await;

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
