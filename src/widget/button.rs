use aplite_renderer::Shape;
use aplite_types::{Rgba, Paint};

use crate::widget_state::WidgetState;
use super::{ViewNode, ViewId, PaintId, Widget, VIEW_STORAGE};

pub fn button() -> Button { Button::new() }

pub struct Button {
    id: ViewId,
    paint_id: PaintId,
    node: ViewNode,
}

impl Button {
    pub fn new() -> Self {
        let (id, paint_id) = VIEW_STORAGE.with(|s| {
            let state = WidgetState::new()
                .with_name("Button")
                .with_size((80, 30));
            let id = s.insert(state);
            let paint_id = s.add_paint(Paint::Color(Rgba::RED));
            (id, paint_id)
        });

        let node = ViewNode::new()
            .with_fill_color(Rgba::RED)
            .with_shape(Shape::RoundedRect);

        Self {
            id,
            paint_id,
            node,
        }
    }

    pub fn state(self, f: impl Fn(&mut WidgetState)) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            let state = tree.get_mut(&self.id).unwrap();
            f(state);
        });
        self
    }
}

impl Widget for Button {
    fn id(&self) -> ViewId {
        self.id
    }

    fn paint_id(&self) -> PaintId {
        self.paint_id
    }

    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}
