use std::cell::RefCell;

use aplite_renderer::Shape;
use aplite_types::{Rgba, CornerRadius, Size};
use aplite_storage::U64Map;

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

type Callbacks = U64Map<ViewId, CallbackStore>;

#[repr(u8)]
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

    fn set_state<F>(self, mut state_fn: F) -> Self
    where
        F: FnMut(&mut WidgetState)
    {
        if let Some(state) = self.node().upgrade() {
            state_fn(&mut state.borrow_mut())
        }
        self
    }

    fn color(self, color: Rgba<u8>) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().background = color.into();
        }
        self
    }

    fn border_color(self, color: Rgba<u8>) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().border_color = color.into();
        }
        self
    }

    fn hover_color(self, color: Rgba<u8>) -> Self {
        let _ = color;
        self
    }

    fn click_color(self, color: Rgba<u8>) -> Self {
        let _ = color;
        self
    }

    fn border_width(self, val: f32) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().border_width = val;
        }
        self
    }

    fn corners(self, corners: CornerRadius) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().corner_radius = corners;
        }
        self
    }

    fn shape(self, shape: Shape) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().shape = shape;
        }
        self
    }

    fn size(self, size: impl Into<Size>) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().rect.set_size(size.into());
        }
        self
    }

    fn dragable(self, value: bool) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().dragable = value;
        }
        VIEW_STORAGE.with(|s| {
            let mut hoverable = s.hoverable.borrow_mut();
            if !hoverable.contains(&self.id()) {
                hoverable.push(self.id());
            }
        });
        self
    }
}

pub(crate) struct CallbackStore(Box<[Option<Box<dyn FnMut()>>; 5]>);

impl Default for CallbackStore {
    fn default() -> Self {
        Self(Box::new([None, None, None, None, None]))
    }
}

impl CallbackStore {
    pub(crate) fn insert(
        &mut self,
        event: WidgetEvent,
        callback: Box<dyn FnMut()>,
    ) -> Option<Box<dyn FnMut()>> {
        self.0[event as usize].replace(callback)
    }

    #[allow(unused)]
    pub(crate) fn get(&self, event: WidgetEvent) -> Option<&Box<dyn FnMut()>> {
        self.0[event as usize].as_ref()
    }

    pub(crate) fn get_mut(&mut self, event: WidgetEvent) -> Option<&mut Box<dyn FnMut()>> {
        self.0[event as usize].as_mut()
    }
}

pub struct CircleWidget {
    node: ViewNode,
}

impl CircleWidget {
    pub fn new() -> Self {
        let node = ViewNode::new()
            .with_name("Circle")
            .with_stroke_width(5.)
            .with_shape(Shape::Circle)
            .with_size((100., 100.));

        Self {
            node,
        }
    }
}

impl Widget for CircleWidget {
    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}
