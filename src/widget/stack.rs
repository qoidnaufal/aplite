use aplite_types::Rgba;
use aplite_renderer::Shape;

use crate::context::layout::Orientation;

use super::{ViewNode, Widget};

pub fn v_stack<F>() -> VStack {
    VStack::new()
}

pub struct VStack {
    node: ViewNode,
}

impl VStack {
    pub fn new() -> Self {
        let node = ViewNode::new()
            .with_name("VStack")
            .with_size((1, 1))
            .with_background_paint(Rgba::TRANSPARENT)
            .with_border_paint(Rgba::TRANSPARENT)
            .with_shape(Shape::Rect);

        Self {
            node,
        }
    }
}

impl Widget for VStack {
    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}

pub fn h_stack<F>() -> HStack {
    HStack::new()
}

pub struct HStack {
    node: ViewNode,
}

impl HStack {
    pub fn new() -> Self {
        let node = ViewNode::new()
            .with_name("VStack")
            .with_orientation(Orientation::Horizontal)
            .with_size((1, 1))
            .with_background_paint(Rgba::TRANSPARENT)
            .with_border_paint(Rgba::TRANSPARENT)
            .with_shape(Shape::Rect);

        Self {
            node,
        }
    }
}

impl Widget for HStack {
    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}
