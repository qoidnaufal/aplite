mod button;
mod image;
mod vstack;

pub use {
    button::*,
    image::*,
    vstack::*,
};
use std::sync::atomic::{AtomicU64, Ordering};
use crate::{
    callback::CALLBACKS,
    color::Rgb,
    shapes::{Shape, ShapeKind},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    fn new() -> Self {
        static NODE_ID: AtomicU64 = AtomicU64::new(0);
        Self(NODE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub trait Widget: View {
    fn on_hover<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> &Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    fn on_click<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> &Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }

    fn on_drag<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> &Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

pub trait View: std::fmt::Debug {
    fn id(&self) -> NodeId;
    fn shape(&self) -> Shape;
    fn children(&self) -> Option<&[(NodeId, Shape)]>;
}

#[derive(Debug)]
pub struct TestCircleWidget {
    id: NodeId,
}

impl TestCircleWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::filled(Rgb::YELLOW, ShapeKind::FilledTriangle)
    }
}

impl View for TestCircleWidget {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn children(&self) -> Option<&[(NodeId, Shape)]> {
        None
    }

    fn shape(&self) -> Shape {
        self.shape()
    }
}

impl View for &TestCircleWidget {
    fn id(&self) -> NodeId {
        (*self).id()
    }

    fn children(&self) -> Option<&[(NodeId, Shape)]> {
        None
    }

    fn shape(&self) -> Shape {
        (*self).shape()
    }
}

impl Widget for TestCircleWidget {}
impl Widget for &TestCircleWidget {}
