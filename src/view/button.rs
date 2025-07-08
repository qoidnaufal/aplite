use aplite_reactive::*;
use aplite_renderer::Shape;
use aplite_types::Rgba;
use crate::context::widget_state::WidgetState;

use super::{Node, IntoView, ViewId, Widget, VIEW_STORAGE};

pub fn button() -> Button { Button::new() }

pub struct Button {
    id: ViewId,
    node: Node,
    state: WidgetState,
}

impl Button {
    pub fn new() -> Self {
        let id = VIEW_STORAGE.with(|s| s.create_entity());
        let node = Node::new()
            .with_fill_color(Rgba::RED)
            .with_shape(Shape::RoundedRect);
        let state = WidgetState::new()
            .with_name("Button")
            .with_size((80, 30));
        state.hoverable.write_untracked(|val| *val = true);

        Self {
            id,
            node,
            state,
        }
    }

    pub fn append_child(self, child: impl IntoView) -> Self {
        VIEW_STORAGE.with(|s| s.append_child(&self.id, child));
        self
    }

    pub fn and(self, sibling: impl IntoView) -> Self {
        VIEW_STORAGE.with(|s| s.add_sibling(&self.id, sibling));
        self
    }
    
    pub fn on_click<F: Fn() + 'static>(self, f: F) -> Self {
        VIEW_STORAGE.with(|s| s.add_on_click(self.id, f));
        self
    }

    pub fn state(mut self, mut f: impl FnMut(&mut WidgetState) + 'static) -> Self {
        f(&mut self.state);
        self
    }
}

impl Widget for Button {
    fn id(&self) -> ViewId {
        self.id
    }

    fn widget_state(&self) -> WidgetState {
        self.state
    }

    fn node(&self) -> Node {
        self.node.clone()
    }
}
