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
}

impl VStack {
    pub fn new() -> Self {
        let (id, paint_id) = VIEW_STORAGE.with(|s| {
            let state = WidgetState::new()
                .with_name("VStack")
                .with_size((1, 1));
            let id = s.insert(state);
            let paint_id = s.add_paint(Paint::from_color(Rgba::TRANSPARENT));
            (id, paint_id)
        });

        let node = ViewNode::new()
            .with_fill_color(Rgba::TRANSPARENT)
            .with_shape(Shape::Rect);

        Self {
            id,
            paint_id,
            node,
        }
    }

    pub fn state(self, f: impl Fn(&mut WidgetState)) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            let state = tree.get_data_mut(&self.id).unwrap();
            f(state);
        });
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
}

impl HStack {
    pub fn new() -> Self {
        let (id, paint_id) = VIEW_STORAGE.with(|s| {
            let state = WidgetState::new()
                .with_orientation(Orientation::Horizontal)
                .with_name("HStack")
                .with_size((1, 1));
            let id = s.insert(state);
            let paint_id = s.add_paint(Paint::Color(Rgba::TRANSPARENT));
            (id, paint_id)
        });

        let node = ViewNode::new()
            .with_fill_color(Rgba::TRANSPARENT)
            .with_shape(Shape::Rect);

        Self {
            id,
            paint_id,
            node,
        }
    }

    pub fn state(self, f: impl Fn(&mut WidgetState)) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            let state = tree.get_data_mut(&self.id).unwrap();
            f(state);
        });
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

    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}
