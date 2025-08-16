use aplite_types::Rgba;
use aplite_renderer::Shape;

use crate::layout::Orientation;

use super::{ViewNode, Widget};

pub fn v_stack<F>() -> VStack {
    VStack::new()
}

pub fn h_stack<F>() -> HStack {
    HStack::new()
}

pub struct VStack {
    node: ViewNode,
    children: Vec<Box<dyn Widget>>,
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
            children: Vec::new(),
        }
    }
}

impl Widget for VStack {
    fn node(&self) -> ViewNode {
        self.node.clone()
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
        Some(&self.children)
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        Some(&mut self.children)
    }
}

pub struct HStack {
    node: ViewNode,
    children: Vec<Box<dyn Widget>>,
}

impl HStack {
    pub fn new() -> Self {
        let node = ViewNode::new()
            .with_name("HStack")
            .with_orientation(Orientation::Horizontal)
            .with_size((1, 1))
            .with_background_paint(Rgba::TRANSPARENT)
            .with_border_paint(Rgba::TRANSPARENT)
            .with_shape(Shape::Rect);

        Self {
            node,
            children: Vec::new(),
        }
    }
}

impl Widget for HStack {
    fn node(&self) -> ViewNode {
        self.node.clone()
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
        Some(&self.children)
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        Some(&mut self.children)
    }
}
