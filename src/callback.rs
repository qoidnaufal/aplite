use std::collections::HashMap;

use crate::tree::NodeId;

pub(crate) struct Action(Box<dyn Fn()>);

pub(crate) struct ActionMut<T>(Box<dyn Fn(&mut T)>);

pub(crate) struct Callback<A> {
    action: HashMap<NodeId, A>,
}

impl<A> Callback<A> {
    pub(crate) fn new() -> Self {
        Self { action: HashMap::new() }
    }
}

impl Callback<Action> {
    pub(crate) fn run(&self, node_id: &NodeId) {
        if let Some(action) = self.get(node_id) {
            action();
        }
    }

    pub(crate) fn insert(&mut self, node_id: NodeId, action: impl Into<Action>) {
        self.action.insert(node_id, action.into());
    }

    pub(crate) fn get(&self, node_id: &NodeId) -> Option<&Action> {
        self.action.get(node_id)
    }
}

impl<T> Callback<ActionMut<T>> {
    pub(crate) fn run(&self, node_id: &NodeId, val: &mut T) {
        if let Some(action) = self.get(node_id) {
            action(val);
        }
    }

    pub(crate) fn insert(&mut self, node_id: NodeId, action: impl Into<ActionMut<T>>) {
        self.action.insert(node_id, action.into());
    }

    pub(crate) fn get(&self, node_id: &NodeId) -> Option<&ActionMut<T>> {
        self.action.get(node_id)
    }
}

impl<F: Fn() + 'static> From<F> for Action {
    fn from(callback: F) -> Self {
        Self(Box::new(callback))
    }
}

impl<T, F: Fn(&mut T) + 'static> From<F> for ActionMut<T> {
    fn from(callback: F) -> Self {
        Self(Box::new(callback))
    }
}

impl std::ops::Deref for Action {
    type Target = Box<dyn Fn()>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::Deref for ActionMut<T> {
    type Target = Box<dyn Fn(&mut T)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod callback {
    use super::*;

    #[test]
    fn action() {
        let mut cb: Callback<Action> = Callback::new();
        let a = || {
            let a = 2;
            let b = 2;
            assert_eq!(a, b);
        };
        cb.insert(NodeId::root(), a);
        cb.run(&NodeId::root());
    }

    #[test]
    fn action_t() {
        let mut cb: Callback<ActionMut<u32>> = Callback::new();
        let a = |num: &mut u32| {
            let b = 2;
            *num += b;
            assert!(*num >= b);
        };
        let mut c = 3u32;
        cb.insert(NodeId::root(), a);
        cb.run(&NodeId::root(), &mut c);
    }
}
