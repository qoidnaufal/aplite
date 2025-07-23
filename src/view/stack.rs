use aplite_types::Rgba;
use aplite_renderer::Shape;

use crate::context::layout::Orientation;
use crate::widget_state::WidgetState;

use super::{ViewNode, ViewId, Widget, VIEW_STORAGE};

pub fn v_stack<F>() -> VStack {
    VStack::new()
}

pub struct VStack {
    id: ViewId,
    node: ViewNode,
    state: WidgetState,
}

impl VStack {
    pub fn new() -> Self {
        let id = VIEW_STORAGE.with(|s| s.create_entity());

        let node = ViewNode::new()
            .with_fill_color(Rgba::TRANSPARENT)
            .with_shape(Shape::Rect);

        let state = WidgetState::new()
            .with_name("VStack")
            .with_size((1, 1));

        Self {
            id,
            node,
            state,
        }
    }

    pub fn state(mut self, f: impl Fn(&mut WidgetState)) -> Self {
        f(&mut self.state);
        self
    }
}

impl Widget for VStack {
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

pub fn h_stack<F>() -> HStack {
    HStack::new()
}

pub struct HStack {
    id: ViewId,
    node: ViewNode,
    state: WidgetState,
}

impl HStack {
    pub fn new() -> Self {
        let id = VIEW_STORAGE.with(|s| s.create_entity());

        let node = ViewNode::new()
            .with_fill_color(Rgba::TRANSPARENT)
            .with_shape(Shape::Rect);

        let state = WidgetState::new()
            .with_orientation(Orientation::Horizontal)
            .with_name("HStack")
            .with_size((1, 1));

        Self {
            id,
            node,
            state,
        }
    }

    pub fn state(mut self, f: impl Fn(&mut WidgetState)) -> Self {
        f(&mut self.state);
        self
    }
}

impl Widget for HStack {
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
