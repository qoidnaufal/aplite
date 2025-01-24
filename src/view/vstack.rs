use crate::Rgb;
use crate::shapes::{Shape, ShapeKind};

use super::{NodeId, View, Widget};

pub fn vstack(child_node: impl Iterator<Item = impl View>) -> VStack {
    VStack::new(child_node)
}

#[derive(Debug)]
pub struct VStack {
    id: NodeId,
    child: Vec<(NodeId, Shape)>,
}

impl VStack {
    fn new(child_node: impl Iterator<Item = impl View>) -> Self {
        let id = NodeId::new();
        let child = child_node.map(|v| (v.id(), v.shape())).collect();
        Self { id, child }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::filled(Rgb::BLACK, ShapeKind::FilledRectangle)
    }

}

impl View for VStack {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn children(&self) -> Option<&[(NodeId, Shape)]> {
        Some(&self.child)
    }

    fn shape(&self) -> Shape {
        self.shape()
    }
}

impl View for &VStack {
    fn id(&self) -> NodeId {
        (*self).id()
    }

    fn children(&self) -> Option<&[(NodeId, Shape)]> {
        Some(&self.child)
    }

    fn shape(&self) -> Shape {
        (*self).shape()
    }
}

impl Widget for VStack {}
impl Widget for &VStack {}
