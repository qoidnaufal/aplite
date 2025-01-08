use std::{cell::RefCell, collections::HashMap};

use crate::{shapes::Shape, NodeId};

thread_local! {
    pub static CALLBACKS: RefCell<Callbacks> = RefCell::new(Callbacks::default());
}

#[derive(Default)]
pub struct Callbacks {
    pub on_click: HashMap<NodeId, Callback>,
    pub on_drag: HashMap<NodeId, Callback>,
    pub on_hover: HashMap<NodeId, Callback>,
}

pub struct Callback(Box<dyn FnMut(&mut Shape) + 'static>);

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

