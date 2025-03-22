use std::{cell::RefCell, collections::HashMap};

use crate::{shapes::Shape, NodeId};

thread_local! {
    pub static CALLBACKS: RefCell<Callbacks> = RefCell::new(Callbacks::default());
}

pub struct Callback(Box<dyn FnMut(&mut Shape) + 'static>);

#[derive(Default)]
pub struct Callbacks {
    pub on_click: HashMap<NodeId, Callback>,
    pub on_drag: HashMap<NodeId, Callback>,
    pub on_hover: HashMap<NodeId, Callback>,
}

impl<F: FnMut(&mut Shape) + 'static> From<F> for Callback {
    fn from(callback: F) -> Self {
        Self(Box::new(callback))
    }
}

impl std::ops::Deref for Callback {
    type Target = Box<dyn FnMut(&mut Shape) + 'static>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Callback {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Callbacks {
    pub fn handle_click(&mut self, node_id: &NodeId, shape: &mut Shape) {
        if let Some(on_click) = self.on_click.get_mut(node_id) {
            on_click(shape);
        }
    }

    pub fn handle_drag(&mut self, node_id: &NodeId, shape: &mut Shape) {
        if let Some(on_drag) = self.on_drag.get_mut(node_id) {
            on_drag(shape)
        }
    }

    pub fn handle_hover(&mut self, node_id: &NodeId, shape: &mut Shape) {
        if let Some(on_hover) = self.on_hover.get_mut(node_id) {
            on_hover(shape)
        }
    }
}
