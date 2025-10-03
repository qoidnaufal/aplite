use aplite_types::Rgba;
use aplite_renderer::Shape;

use crate::layout::Orientation;
use crate::state::WidgetState;

use super::{Widget, Children};

pub fn v_stack<F>() -> VStack {
    VStack::new()
}

pub fn h_stack<F>() -> HStack {
    HStack::new()
}

pub struct VStack {
    state: WidgetState,
    children: Children,
}

impl VStack {
    pub fn new() -> Self {
        let state = WidgetState::default()
            .with_size(1, 1)
            .with_background_paint(Rgba::TRANSPARENT)
            .with_border_paint(Rgba::TRANSPARENT)
            .with_shape(Shape::Rect);

        let mut children = Children::new();
        children.orientation(Orientation::Vertical);

        Self {
            state,
            children,
        }
    }
}

impl Widget for VStack {
    fn state(&self) -> &WidgetState {
        &self.state
    }

    fn children(&self) -> Option<&Children> {
        Some(&self.children)
    }
}

pub struct HStack {
    state: WidgetState,
    children: Children,
}

impl HStack {
    pub fn new() -> Self {
        let state = WidgetState::default()
            .with_size(1, 1)
            .with_background_paint(Rgba::TRANSPARENT)
            .with_border_paint(Rgba::TRANSPARENT)
            .with_shape(Shape::Rect);

        let mut children = Children::new();
        children.orientation(Orientation::Horizontal);

        Self {
            state,
            children,
        }
    }
}

impl Widget for HStack {
    fn state(&self) -> &WidgetState {
        &self.state
    }

    fn children(&self) -> Option<&Children> {
        Some(&self.children)
    }
}
