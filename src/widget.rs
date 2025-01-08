mod button;
mod image;

pub use {
    button::Button,
    image::Image,
};
use std::sync::atomic::{AtomicU64, Ordering};
use math::{Size, Vector2};
use crate::{
    callback::CALLBACKS,
    color::Rgb,
    shapes::{Shape, FilledShape},
};

thread_local! {
    pub static NODE_ID: AtomicU64 = const { AtomicU64::new(0) };
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    fn new() -> Self {
        Self(NODE_ID.with(|i| i.fetch_add(1, Ordering::Relaxed)))
    }
}

pub trait Widget: std::fmt::Debug {
    fn id(&self) -> NodeId;
    fn shape(&self) -> Shape;
}

#[derive(Debug, Clone, Copy)]
pub struct TestWidget {
    id: NodeId,
}

impl TestWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::new(Vector2::new(), Size::new(500, 500), Rgb::RED, FilledShape::FilledTriangle)
    }

    pub fn on_hover<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        *self
    }

    pub fn on_click<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        *self
    }

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        *self
    }
}

impl Widget for TestWidget {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn shape(&self) -> Shape {
        self.shape()
    }
}

#[derive(Debug, Clone, Copy)]
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
        Shape::new(Vector2::new(), Size::new(500, 500), Rgb::BLACK, FilledShape::FilledCircle)
    }

    pub fn on_hover<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        *self
    }

    pub fn on_click<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        *self
    }

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        *self
    }
}

impl Widget for TestCircleWidget {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn shape(&self) -> Shape {
        self.shape()
    }
}
