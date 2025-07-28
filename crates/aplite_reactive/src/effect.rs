use std::rc::Rc;
use std::cell::RefCell;
use aplite_future::{
    Channel,
    Sender,
    Executor,
};

use crate::graph::{EffectId, GRAPH};
use crate::subscriber::{Subscriber, ToAnySubscriber};

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

        let inner = EffectInner { sender: tx };
        let any_subscriber = inner.to_any_subscriber();
        let id = GRAPH.with(|graph| {
            graph.subscribers
                .borrow_mut()
                .insert(any_subscriber)
        });

        Executor::spawn_local(async move {
            let value = Rc::new(RefCell::new(None::<R>));
            while rx.recv().await.is_some() {
                eprintln!("[NOTIFIED] {id:?} is running the function");
                let prev_scope = GRAPH.with(|graph| {
                    graph.current
                        .borrow_mut()
                        .replace(id)
                });

                let prev_value = value.borrow_mut().take();
                let new_val = f(prev_value);
                *value.borrow_mut() = Some(new_val);

                GRAPH.with(|graph| *graph.current.borrow_mut() = prev_scope);
            }
        });

        Self { id }
    }
}

pub(crate) struct EffectInner {
    pub(crate) sender: Sender,
}

impl Subscriber for EffectInner {
    fn notify(&self) {
        self.sender.notify();
    }
}

#[cfg(test)]
mod effect_test {
    use std::rc::Rc;
    use std::cell::RefCell;
    use aplite_future::{Runtime, Executor};
    // use aplite_future::sleep;
    use crate::signal::Signal;
    use crate::reactive_traits::*;
    use super::*;

    #[test]
    fn effect() {
        let rt = Runtime::init_local();
        rt.spawn_local(async {
            let use_last = Signal::new(false);
            let (first, set_first) = Signal::split("Dario");
            let (last, set_last) = Signal::split("");

            let name = Rc::new(RefCell::new(String::new()));
            let set_name = Rc::clone(&name);

            Effect::new(move |_| {
                if use_last.get() {
                    *set_name.borrow_mut() = first.get().to_string() + " " + last.get();
                } else {
                    *set_name.borrow_mut() = first.with(|n| n.to_string());
                }
            });

            Executor::spawn_local(async move {
                // sleep(100).await;
                eprintln!("-- setting first name to Mario, SHOULD RERUN");
                set_first.set("Mario");

                eprintln!("\n-- setting last name to Ballotelli");
                set_last.set("Ballotelli");
                // assert_eq!("Mario", name.borrow().as_str());

                eprintln!("\n-- set to use last name, SHOULD RERUN");
                use_last.set(true);
                // assert_eq!("Mario Ballotelli", name.borrow().as_str());

                eprintln!("\n-- set to use first only, SHOULD RERUN");
                use_last.set(false);
                // // assert_eq!("Mario", name.borrow().as_str());

                // set_last.set("Gomez");
                // // assert_eq!("Mario", name.borrow().as_str());

                // set_last.set("Bros");
                // // assert_eq!("Mario", name.borrow().as_str());

                eprintln!("\n-- setting last name to Kempes");
                set_last.set("Kempes");
                // // assert_eq!("Mario", name.borrow().as_str());

                eprintln!("\n-- set to use last name, SHOULD RERUN");
                use_last.set(true);
                // assert_eq!("Mario Kempes", name.borrow().as_str());
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

    //         sleep(1).await;

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
