use aplite_types::{Rgba, Paint};
use aplite_renderer::Shape;

use crate::context::layout::Orientation;
use crate::widget_state::WidgetState;

use super::{ViewNode, ViewId, PaintId, Widget, VIEW_STORAGE};

pub fn v_stack<F>() -> VStack {
    VStack::new()
}

pub struct VStack {
    id: ViewId,
    paint_id: PaintId,
    node: ViewNode,
    state: WidgetState,
}

impl VStack {
    pub fn new() -> Self {
        let (id, paint_id) = VIEW_STORAGE.with(|s| {
            let id = s.create_entity();
            let paint_id = s.add_paint(Paint::from_color(Rgba::TRANSPARENT));
            (id, paint_id)
        });

        let node = ViewNode::new()
            .with_fill_color(Rgba::TRANSPARENT)
            .with_shape(Shape::Rect);

        let state = WidgetState::new()
            .with_name("VStack")
            .with_size((1, 1));

        Self {
            id,
            paint_id,
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

pub fn h_stack<F>() -> HStack {
    HStack::new()
}

pub struct HStack {
    id: ViewId,
    paint_id: PaintId,
    node: ViewNode,
    state: WidgetState,
}

impl HStack {
    pub fn new() -> Self {
        let (id, paint_id) = VIEW_STORAGE.with(|s| {
            let id = s.create_entity();
            let paint_id = s.add_paint(Paint::Color(Rgba::TRANSPARENT));
            (id, paint_id)
        });

        let node = ViewNode::new()
            .with_fill_color(Rgba::TRANSPARENT)
            .with_shape(Shape::Rect);

        let state = WidgetState::new()
            .with_orientation(Orientation::Horizontal)
            .with_name("HStack")
            .with_size((1, 1));

        Self {
            id,
            paint_id,
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
