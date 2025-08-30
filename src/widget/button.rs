use aplite_renderer::Shape;
use super::{ViewNode, Widget};

pub fn button() -> Button { Button::new() }

pub struct Button {
    node: ViewNode,
}

impl Button {
    pub fn new() -> Self {
        let node = ViewNode::default()
            .with_shape(Shape::RoundedRect)
            .hoverable()
            .with_size((80, 30));

        Self {
            node,
        }
    }
}

impl Widget for Button {
    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}
