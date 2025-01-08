use crate::{
    color::Rgb,
    shapes::{Shape, FilledShape},
};
use math::{Size, Vector2};

use super::{NodeId, Widget, CALLBACKS};

#[derive(Debug, Clone, Copy)]
pub struct Button {
    id: NodeId,
}

impl Button {
    pub fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::new(Vector2::new(), Size::new(500, 500), Rgb::RED, FilledShape::FilledRectangle)
    }

    pub fn on_hover<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> &Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    pub fn on_click<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> &Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> &Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl Widget for Button {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn shape(&self) -> Shape {
        self.shape()
    }
}

impl Widget for &Button {
    fn id(&self) -> NodeId {
        (*self).id()
    }

    fn shape(&self) -> Shape {
        (*self).shape()
    }
}
