use crate::{
    color::Rgb,
    shapes::{Shape, ShapeKind},
};
use super::{NodeId, View, Widget};

pub fn button() -> Button {
    Button::new()
}

#[derive(Debug)]
pub struct Button {
    id: NodeId,
}

impl Button {
    fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::filled(Rgb::RED, ShapeKind::FilledRectangle)
    }
}

impl View for Button {
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

impl View for &Button {
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

impl Widget for Button {}
impl Widget for &Button {}
