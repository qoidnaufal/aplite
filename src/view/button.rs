use aplite_reactive::*;
use aplite_renderer::Shape;
use aplite_types::{Rgba, Paint};

use crate::widget_state::WidgetState;
use super::{ViewNode, ViewId, PaintId, Widget, VIEW_STORAGE};

pub fn button() -> Button { Button::new() }

pub struct Button {
    id: ViewId,
    paint_id: PaintId,
    node: ViewNode,
    state: WidgetState,
}

impl Button {
    pub fn new() -> Self {
        let (id, paint_id) = VIEW_STORAGE.with(|s| {
            let id = s.create_entity();
            let paint_id = s.add_paint(Paint::Color(Rgba::RED));
            (id, paint_id)
        });

        let node = ViewNode::new()
            .with_fill_color(Rgba::RED)
            .with_shape(Shape::RoundedRect);

        let state = WidgetState::new()
            .with_name("Button")
            .with_size((80, 30));

        Self {
            id,
            paint_id,
            node,
            state,
        }
    }
    
    pub fn on_click<F: Fn() + 'static>(self, f: F) -> Self {
        let trigger = self.state.trigger_callback;
        Effect::new(move |_| {
            if trigger.get() {
                f();
            }
        });
        self
    }

    pub fn state(mut self, f: impl Fn(&mut WidgetState)) -> Self {
        f(&mut self.state);
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

    fn widget_state(&self) -> &WidgetState {
        &self.state
    }

    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}
