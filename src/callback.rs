use std::{cell::RefCell, collections::HashMap};

use crate::context::NodeId;

thread_local! {
    pub static CALLBACKS: RefCell<Callbacks> = RefCell::new(Callbacks::default());
}

pub struct Action(Box<dyn Fn() + 'static>);

#[derive(Default)]
pub struct Callbacks {
    action: HashMap<NodeId, Action>,
}

impl Callbacks {
    pub(crate) fn insert(&mut self, node_id: NodeId, action: impl Into<Action>) {
        self.action.insert(node_id, action.into());
    }

    pub(crate) fn get(&self, node_id: &NodeId) -> Option<&Action> {
        self.action.get(node_id)
    }

    pub(crate) fn run(&self, node_id: &NodeId) {
        if let Some(action) = self.get(node_id) {
            action();
        }
    }
}

impl<F: Fn() + 'static> From<F> for Action {
    fn from(callback: F) -> Self {
        Self(Box::new(callback))
    }
}

impl std::ops::Deref for Action {
    type Target = Box<dyn Fn() + 'static>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Action {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
