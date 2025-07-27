use std::cell::RefCell;
use std::sync::{Arc, RwLock};
use aplite_future::{
    channel,
    Sender,
    Receiver,
    Executor,
    StreamExt
};

use crate::graph::{EffectId, GRAPH};
use crate::subscriber::Subscriber;

/// [`Effect`] is a scope to synchronize the reactive node (eg: [`Signal`](crate::signal::Signal)) with anything.
/// I remember Greg Johnstone, the creator of Leptos, said that an this should only be used
/// to synchronize reactive node with non reactive API, but so far I don't think I encounter any error.
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
    pub(crate) id: EffectId,
}

impl Effect {
    pub fn new<F, R>(mut f: F) -> Self
    where
        F: FnMut(Option<&R>) -> R + Send + 'static,
        R: 'static + Send + Sync,
    {
        let (tx, mut rx) = channel();

        Executor::spawn_local({
            let value = Arc::new(RwLock::new(None::<R>));

            async move {
                while rx.next().await.is_some() {
                    let prev = value.read().unwrap();
                    let new_val = f(prev.as_ref());
                    *value.write().unwrap() = Some(new_val);
                }
            }
        });

        GRAPH.with(|rt| rt.create_effect(EffectInner::new(tx)))
    }

    pub(crate) fn id(&self) -> &EffectId {
        &self.id
    }
}

pub(crate) struct EffectInner {
    pub(crate) sender: Sender,
}

impl EffectInner {
    pub(crate) fn new(sender: Sender) -> Self {
        Self {
            sender,
        }
    }

    pub(crate) fn notify(&self) {
        self.sender.notify();
    }
}

impl Subscriber for RefCell<EffectInner> {
    fn notify(&self) {
        let inner = self.borrow();
        inner.notify();
    }
}

struct InnerAlt {
    tx: Sender,
}
