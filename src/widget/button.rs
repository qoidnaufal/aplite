use aplite_renderer::Shape;
use super::{NodeRef, Widget};

pub fn button() -> Button { Button::new() }

pub struct Button {
    node: NodeRef,
}

impl Button {
    pub fn new() -> Self {
        let node = NodeRef::default()
            .with_shape(Shape::RoundedRect)
            .hoverable()
            .with_size((80, 30));

        Self {
            node,
        }
    }
}

impl Widget for Button {
    fn node_ref(&self) -> NodeRef {
        self.node.clone()
    }
}
