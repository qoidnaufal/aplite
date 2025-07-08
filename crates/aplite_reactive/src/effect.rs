use std::cell::RefCell;

use crate::graph::{EffectId, GRAPH};
use crate::subscriber::Subscriber;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Effect {
    id: EffectId,
}

impl Effect {
    pub fn new<F, R>(f: F) -> Self
    where
        F: FnMut(Option<R>) -> R + 'static,
        R: 'static,
    {
        GRAPH.with(|rt| rt.create_effect(f))
    }

    pub(crate) fn with_id(id: EffectId) -> Self {
        Self { id }
    }

    pub(crate) fn id(&self) -> &EffectId {
        &self.id
    }
}

pub(crate) struct EffectInner<R> {
    pub(crate) value: Option<R>,
    pub(crate) f: Box<dyn FnMut(Option<R>) -> R>,
}

impl<R> EffectInner<R> {
    pub(crate) fn new<F>(f: F) -> Self
    where
        F: FnMut(Option<R>) -> R + 'static,
        R: 'static,
    {
        Self {
            value: None,
            f: Box::new(f),
        }
    }

    pub(crate) fn run(&mut self) {
        let old_val = self.value.take();
        let new_val = (self.f)(old_val);
        self.value = Some(new_val);
    }
}

impl<R> Subscriber for RefCell<EffectInner<R>> {
    fn invoke(&self) {
        let mut inner = self.borrow_mut();
        inner.run();
    }
}
