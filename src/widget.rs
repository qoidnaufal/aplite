use std::cell::RefCell;
use std::collections::HashMap;

use aplite_renderer::Shape;
use aplite_storage::Map;

use crate::state::WidgetState;
use crate::view::{
    IntoView,
    ViewId,
    ViewNode,
    VIEW_STORAGE,
};

mod button;
mod image;
mod stack;

pub use {
    button::*,
    image::*,
    stack::*,
};

thread_local! {
    pub(crate) static CALLBACKS: RefCell<Callbacks> = RefCell::new(Default::default());
}

type Callbacks = HashMap<ViewId, WidgetCallback>;

struct WidgetCallback(Map<WidgetEvent, Box<dyn FnMut()>>);

#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetEvent {
    Hover,
    LeftClick,
    RightClick,
    Drag,
    Input,
}

/// main building block to create a renderable component
pub trait Widget {
    fn node(&self) -> ViewNode;

    fn id(&self) -> ViewId {
        self.node().0
    }
}

impl<T> WidgetExt for T where T: Widget + Sized {}

// TODO: is immediately calculate the size here a good idea?
pub trait WidgetExt: Widget + Sized {
    fn child(self, child: impl IntoView) -> Self {
        VIEW_STORAGE.with(|s| s.append_child(&self.id(), child));
        self
    }

    fn and(self, sibling: impl IntoView) -> Self {
        VIEW_STORAGE.with(|s| s.add_sibling(&self.id(), sibling));
        self
    }

    fn on<F>(self, event: WidgetEvent, f: F) -> Self
    where
        F: FnMut() + 'static,
    {
        CALLBACKS.with(|cell| {
            let mut storage = cell.borrow_mut();
            let callbacks = storage.entry(self.id()).or_default();
            callbacks.insert(event, Box::new(f));
        });
        self
    }

    fn state<F>(self, mut state_fn: F) -> Self
    where
        F: FnMut(&mut WidgetState)
    {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.id()) {
                state_fn(state);
            }
        });
        self
    }
}

impl Default for WidgetCallback {
    fn default() -> Self {
        Self(Map::with_capacity(5))
    }
}

impl std::ops::Deref for WidgetCallback {
    type Target = Map<WidgetEvent, Box<dyn FnMut()>>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl std::ops::DerefMut for WidgetCallback {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::hash::Hash for WidgetEvent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(*self as u64);
    }
}

pub struct CircleWidget {
    node: ViewNode,
}

impl CircleWidget {
    pub fn new() -> Self {
        let node = ViewNode::new()
            .with_name("Circle")
            .with_stroke_width(5)
            .with_shape(Shape::Circle)
            .with_size((100., 100.));

        Self {
            node,
        }
    }
}

impl Widget for CircleWidget {
    fn node(&self) -> ViewNode {
        self.node
    }
}
