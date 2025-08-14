use aplite_renderer::Shape;
use super::{ViewNode, Widget, WidgetId};

pub fn button() -> Button { Button::new() }

pub struct Button {
    id: WidgetId,
    node: ViewNode,
}

impl Button {
    pub fn new() -> Self {
        let node = ViewNode::new()
            .with_name("Button")
            .with_shape(Shape::RoundedRect)
            .hoverable()
            .with_size((80, 30));

        Self {
            id: WidgetId::new(),
            node,
        }
    }
}

impl Widget for Button {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}
