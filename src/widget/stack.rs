use aplite_types::Rgba;
use aplite_renderer::Shape;

use crate::layout::Orientation;

use super::{ViewNode, Widget, WidgetId};

pub fn v_stack<F>() -> VStack {
    VStack::new()
}

pub fn h_stack<F>() -> HStack {
    HStack::new()
}

pub struct VStack {
    id: WidgetId,
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
            id: WidgetId::new(),
            node,
            children: Vec::new(),
        }
    }
}

impl Widget for VStack {
    fn id(&self) -> WidgetId {
        self.id
    }

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
    id: WidgetId,
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
            id: WidgetId::new(),
            node,
            children: Vec::new(),
        }
    }
}

impl Widget for HStack {
    fn id(&self) -> WidgetId {
        self.id
    }

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
