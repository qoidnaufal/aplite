use std::sync::{Arc, RwLock};
use aplite_future::{
    Channel,
    Sender,
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
    node: ReactiveNode<Arc<Reactor>>,
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
        let node = GRAPH.with(|graph| {
            graph.insert(reactor)
        });

        Executor::spawn_local(async move {
            let value = Arc::new(RwLock::new(None::<R>));

            while rx.recv().await.is_some() {
                #[cfg(test)] eprintln!("\n[NOTIFIED]      : {node:?}");
                let prev_scope = GRAPH.with(|graph| {
                    let stored = graph.get(&node).unwrap();
                    let reactor = stored.downcast_ref::<Arc<Reactor>>().unwrap();
                    reactor.clear_source();
                    graph.swap_current(Some(Arc::clone(reactor).to_any_subscriber()))
                });

                let mut lock = value.write().unwrap();
                let prev_value = lock.take();
                let new_val = f(prev_value);
                *lock = Some(new_val);

                GRAPH.with(|graph| graph.swap_current(prev_scope));
            }
        });

        Self { node }
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

impl Reactive for Reactor {
    fn dirty(&self) {
        self.notify();
    }

    fn subscribe(&self) {}

    fn unsubscribe(&self) {
        self.clear_source();
    }
}

impl Reactive for Arc<Reactor> {
    fn dirty(&self) {
        self.as_ref().dirty();
    }

    fn subscribe(&self) {}

    fn unsubscribe(&self) {
        self.as_ref().unsubscribe();
    }
}

impl std::fmt::Debug for Reactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EffectInner")
            .field("id", &std::any::TypeId::of::<Self>())
            .finish()
    }
}

#[cfg(test)]
mod effect_test {
    use std::rc::Rc;
    use std::cell::RefCell;
    use aplite_future::{Runtime, Executor};
    use aplite_future::sleep;
    use crate::signal::Signal;
    use crate::reactive_traits::*;
    use super::*;

    #[test]
    fn effect() {
        let rt = Runtime::init();

        let use_last = Signal::new(false);
        let (first, set_first) = Signal::split("Dario");
        let (last, set_last) = Signal::split("");

        rt.spawn_local(async move {
            let name = Rc::new(RefCell::new(String::new()));
            let set_name = Rc::clone(&name);

            Effect::new(move |_| {
                if use_last.get() {
                    *set_name.borrow_mut() = first.get().to_string() + " " + last.get();
                } else {
                    *set_name.borrow_mut() = first.read(|n| n.to_string());
                }
            });

            Executor::spawn_local(async move {
                // eprintln!("-- setting first name to Mario, SHOULD RERUN");
                set_first.set("Mario");
                sleep(1000).await;

                // eprintln!("-- setting last name to Ballotelli");
                set_last.set("Ballotelli");
                sleep(1000).await;
                assert_eq!("Mario", name.borrow().as_str());

                // eprintln!("\n-- set to use last name, SHOULD RERUN");
                use_last.set(true);
                sleep(1000).await;
                assert_eq!("Mario Ballotelli", name.borrow().as_str());

                // eprintln!("-- set to use first only, SHOULD RERUN");
                use_last.set(false);
                sleep(1000).await;
                assert_eq!("Mario", name.borrow().as_str());

                // eprintln!("-- setting last name to Gomez");
                set_last.set("Gomez");
                sleep(1000).await;
                assert_eq!("Mario", name.borrow().as_str());

                // eprintln!("\n-- setting last name to Bros");
                set_last.set("Bros");
                sleep(1000).await;
                assert_eq!("Mario", name.borrow().as_str());

                // eprintln!("\n-- setting last name to Kempes");
                set_last.set("Kempes");
                sleep(1000).await;
                assert_eq!("Mario", name.borrow().as_str());

                // eprintln!("\n-- set to use last name, SHOULD RERUN");
                use_last.set(true);
                sleep(1000).await;
                assert_eq!("Mario Kempes", name.borrow().as_str());
            });
        });
        rt.run();
    }

    // #[test]
    // fn sleep_count() {
    //     let rt = Runtime::init_local();

    //     rt.spawn_local(async move {
    //         let (counter, set_counter) = Signal::split(0);

    //         Effect::new(move |_| {
    //             eprintln!("rerun: {}", counter.get());
    //         });

    //         Executor::spawn_local(async move {
    //             for i in 0..4 {
    //                 sleep(1000).await;
    //                 set_counter.set(i + 1);
    //             }
    //         });
    //     });

    //     rt.run();
    // }

    // #[test]
    // fn child_effect() {
    //     let rt = Runtime::init_local();
    //     rt.spawn_local(async {
    //         let (check, set_check) = Signal::split(false);
    //         let (outer_name, set_outer_name) = Signal::split("Steve");

    //         let someone = Rc::new(RefCell::new(String::new()));
    //         let outer_one = Rc::clone(&someone);

    //         Effect::new(move |_| {
    //             let (inner_name, set_inner_name) = Signal::split("");
    //             let inner_one = Rc::clone(&outer_one);

    //             Effect::new(move |_| {
    //                 if check.get() {
    //                     inner_name.with(|n| *inner_one.borrow_mut() = n.to_string());
    //                 }
    //             });

    //             if check.get() {
    //                 set_inner_name.set("Oscar");
    //             } else {
    //                 *outer_one.borrow_mut() = outer_name.get().to_string();
    //             }
    //         });

    //         sleep(1000).await;

    //         assert_eq!(someone.borrow().as_str(), "Steve");

    //         set_check.set(true);
    //         assert_eq!(someone.borrow().as_str(), "Oscar");

    //         set_outer_name.set("Douglas");

    //         set_check.set(false);
    //         assert_eq!(someone.borrow().as_str(), "Douglas");

    //         set_check.set(true);
    //         assert_eq!(someone.borrow().as_str(), "Oscar");
    //     });
    //     rt.run();
    // }
}
