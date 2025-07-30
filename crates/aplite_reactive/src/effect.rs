use std::rc::Rc;
use std::cell::RefCell;
use aplite_future::{
    Channel,
    Sender,
    Executor,
};

use crate::graph::{ReactiveId, EffectId, GRAPH};
use crate::subscriber::{Subscriber, ToAnySubscriber, AnySubscriber};

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
    id: EffectId,
}

impl Effect {
    pub fn new<F, R>(mut f: F) -> Self
    where
        F: FnMut(Option<R>) -> R + 'static,
        R: 'static,
    {
        let (tx, mut rx) = Channel::new();
        tx.notify();

        let inner = EffectInner::new(tx);
        let any_subscriber = inner.to_any_subscriber();
        let weak_subscriber = any_subscriber.downgrade();
        let id = GRAPH.with(|graph| {
            graph.subscribers
                .borrow_mut()
                .insert(any_subscriber)
        });

        Executor::spawn_local(async move {
            let value = Rc::new(RefCell::new(None::<R>));

            while rx.recv().await.is_some() {
                // eprintln!("[NOTIFIED] {id:?} is running the function");
                weak_subscriber.clear_source();

                let prev_scope = GRAPH.with(|graph| graph.swap_current(Some(weak_subscriber.clone())));

                let prev_value = value.borrow_mut().take();
                let new_val = f(prev_value);
                *value.borrow_mut() = Some(new_val);

                let _ = GRAPH.with(|graph| graph.swap_current(prev_scope));
            }
        });

        Self { id }
    }
}

pub(crate) struct EffectInner {
    pub(crate) sender: Sender,
    pub(crate) source: RefCell<Vec<ReactiveId>>,
}

impl EffectInner {
    fn new(sender: Sender) -> Self {
        Self {
            sender,
            source: RefCell::new(Vec::new()),
        }
    }
}

impl Subscriber for EffectInner {
    fn notify(&self) {
        self.sender.notify();
    }

    fn add_source(&self, source: ReactiveId) {
        self.source.borrow_mut().push(source);
    }

    fn clear_source(&self) {
        let mut sources = self.source.borrow_mut();
        let drained_sources = sources.drain(..);
        GRAPH.with(|graph| drained_sources
            .into_iter()
            .for_each(|id| graph.untrack(&id))
        );
    }
}

impl ToAnySubscriber for EffectInner {
    fn to_any_subscriber(self) -> AnySubscriber {
        AnySubscriber::new(self)
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

    #[test]
    fn effect() {
        let rt = Runtime::init_local();

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
