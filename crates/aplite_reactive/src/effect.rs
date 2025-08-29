use std::sync::{Arc, RwLock};
use aplite_future::{
    Channel,
    Sender,
    Receiver,
    Executor,
};

use crate::graph::{Node, Graph};
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
    node: Node<Arc<Scope>>,
}

impl Effect {
    pub fn new<F, R>(f: F) -> Self
    where
        F: FnMut(Option<R>) -> R + 'static,
        R: 'static,
    {
        let (tx, rx) = Channel::new();
        Self::with_scope(Scope::new(tx), rx, f)
    }

    pub fn with_scope<F, R>(scope: Scope, mut rx: Receiver, mut f: F) -> Self
    where
        F: FnMut(Option<R>) -> R + 'static,
        R: 'static,
    {
        scope.sender.notify();
        let scope = Arc::new(scope);
        let any_subscriber = scope.to_any_subscriber();
        let node = Graph::insert(scope);

        Executor::spawn(async move {
            let mut value = None::<R>;
            let scope = any_subscriber;

            while rx.recv().await.is_some() {
                let prev_scope = Graph::set_scope(Some(scope.clone()));

                scope.clear_source();

                let prev_value = value.take();
                let new_val = f(prev_value);
                value = Some(new_val);

                let _ = Graph::set_scope(prev_scope);

                if scope.source_count() == 0 { break }
            }

            Graph::with_mut(|graph| graph.storage.remove(node.id));
        });

        Self { node }
    }
}

pub struct Scope {
    pub(crate) sender: Sender,
    pub(crate) source: RwLock<Vec<AnySource>>,
}

unsafe impl Send for Scope {}
unsafe impl Sync for Scope {}

impl Scope {
    pub fn new(sender: Sender) -> Self {
        Self {
            sender,
            source: RwLock::new(Vec::new()),
        }
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        self.sender.close();
        self.source.write().unwrap().clear();
    }
}

impl Subscriber for Scope {
    fn add_source(&self, source: AnySource) {
        let mut sources = self.source.write().unwrap();
        if !sources.contains(&source) { sources.push(source) }
    }

    fn clear_source(&self) {
        let mut sources = self.source.write().unwrap();
        sources.clear();
        // let drained_sources = sources.drain(..);
        // drained_sources.into_iter().for_each(|source| source.untrack())
    }

    fn source_count(&self) -> usize {
        let source = self.source.read().unwrap();
        source.len()
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
    fn to_any_subscriber(&self) -> AnySubscriber {
        AnySubscriber::new(Arc::downgrade(self))
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

// impl std::fmt::Debug for Scope {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Scope")
//             .field("source_count", &self.source.read().map(|s| s.len()).unwrap_or_default())
//             .finish()
//     }
// }

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
        Executor::init();

        let use_last = Signal::new(false);
        let (first, set_first) = Signal::split("Dario");
        let (last, set_last) = Signal::split("");

        let name = Rc::new(RefCell::new(String::new()));
        let set_name = Rc::clone(&name);

        Executor::spawn(async move {
            Effect::new(move |_| {
                if use_last.get() {
                    *set_name.borrow_mut() = first.get().to_string() + " " + last.get();
                } else {
                    *set_name.borrow_mut() = first.with(|n| n.to_string());
                }
            });

            Effect::new(move |_| eprintln!("last name: {}", last.get_untracked()));
        });

        Executor::spawn(async move {
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

        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
