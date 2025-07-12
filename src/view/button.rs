use aplite_reactive::*;
use aplite_renderer::Shape;
use aplite_types::Rgba;

use crate::widget_state::WidgetState;
use super::{ViewNode, ViewId, Widget, VIEW_STORAGE};

pub fn button() -> Button { Button::new() }

pub struct Button {
    id: ViewId,
    node: ViewNode,
    state: WidgetState,
}

impl Button {
    pub fn new() -> Self {
        let id = VIEW_STORAGE.with(|s| s.create_entity());

        let node = ViewNode::new()
            .with_fill_color(Rgba::RED)
            .with_shape(Shape::RoundedRect);

        let state = WidgetState::new()
            .with_name("Button")
            .with_size((80, 30));
        state.hoverable.set(true);

        Self {
            id,
            node,
            state,
        }
    }
    
    pub fn on_click<F: Fn() + 'static>(self, f: F) -> Self {
        let trigger = self.state.trigger_callback;
        Effect::new(move |_| {
            if trigger.get() {
                f();
                trigger.set_untracked(false);
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

    fn widget_state(&self) -> &WidgetState {
        &self.state
    }

    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}
