use aplite_types::Rgba;
use aplite_renderer::Shape;

use crate::layout::{LayoutRules, Orientation};
use crate::state::WidgetState;

use super::{WidgetId, Widget, ParentWidget, ENTITY_MANAGER};

pub fn v_stack<F>() -> VStack {
    VStack::new()
}

pub fn h_stack<F>() -> HStack {
    HStack::new()
}

pub struct VStack {
    id: WidgetId,
    state: WidgetState,
    layout_rules: LayoutRules,
}

impl VStack {
    pub fn new() -> Self {
        let state = WidgetState::default()
            .with_size(1, 1)
            .with_background_paint(Rgba::TRANSPARENT)
            .with_border_paint(Rgba::TRANSPARENT)
            .with_shape(Shape::Rect);

        let mut layout_rules = LayoutRules::default();
        layout_rules.orientation = Orientation::Vertical;

        Self {
            id: ENTITY_MANAGER.with_borrow_mut(|m| m.create()),
            state,
            layout_rules,
        }
    }
}

impl Widget for VStack {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn state(&mut self) -> &mut WidgetState {
        &mut self.state
    }
}

impl ParentWidget for VStack {
    fn layout_rules(&mut self) -> &mut LayoutRules {
        &mut self.layout_rules
    }
}

pub struct HStack {
    id: WidgetId,
    state: WidgetState,
    layout_rules: LayoutRules,
}

impl HStack {
    pub fn new() -> Self {
        let state = WidgetState::default()
            .with_size(1, 1)
            .with_background_paint(Rgba::TRANSPARENT)
            .with_border_paint(Rgba::TRANSPARENT)
            .with_shape(Shape::Rect);

        let mut layout_rules = LayoutRules::default();
        layout_rules.orientation = Orientation::Horizontal;

        Self {
            id: ENTITY_MANAGER.with_borrow_mut(|m| m.create()),
            state,
            layout_rules,
        }
    }
}

impl Widget for HStack {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn state(&mut self) -> &mut WidgetState {
        &mut self.state
    }
}

impl ParentWidget for HStack {
    fn layout_rules(&mut self) -> &mut LayoutRules {
        &mut self.layout_rules
    }
}
