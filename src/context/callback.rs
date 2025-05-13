use std::collections::HashMap;

use crate::tree::NodeId;

use super::Properties;

pub(crate) struct Action(Box<dyn Fn()>);

pub(crate) struct StyleFn(Box<dyn Fn(&mut Properties)>);

pub(crate) struct Callback {
    action: HashMap<NodeId, Action>,
    style_fn: HashMap<NodeId, StyleFn>,
}

impl Callback {
    pub(crate) fn new() -> Self {
        Self {
            action: HashMap::new(),
            style_fn: HashMap::new(),
        }
    }
}

impl Callback {
    pub(crate) fn action(&self, node_id: &NodeId) {
        if let Some(action) = self.action.get(node_id) {
            action();
        }
    }

    pub(crate) fn style_fn(&self, node_id: &NodeId, prop: &mut Properties) {
        if let Some(style_fn) = self.style_fn.get(node_id) {
            style_fn(prop);
        }
    }

    pub(crate) fn insert_action(&mut self, node_id: NodeId, action: impl Into<Action>) {
        self.action.insert(node_id, action.into());
    }

    pub(crate) fn insert_style_fn(&mut self, node_id: NodeId, style_fn: impl Into<StyleFn>) {
        self.style_fn.insert(node_id, style_fn.into());
    }
}

impl<F: Fn() + 'static> From<F> for Action {
    fn from(callback: F) -> Self {
        Self(Box::new(callback))
    }
}

impl<F: Fn(&mut Properties) + 'static> From<F> for StyleFn {
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

impl std::ops::Deref for StyleFn {
    type Target = Box<dyn Fn(&mut Properties)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
