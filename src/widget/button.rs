use aplite_renderer::Shape;
use super::{Widget, WidgetId, ENTITY_MANAGER};
use crate::state::WidgetState;

pub fn button() -> Button { Button::new() }

pub struct Button {
    id: WidgetId,
    state: WidgetState,
}

impl Button {
    pub fn new() -> Self {
        let id = ENTITY_MANAGER.with_borrow_mut(|m| m.create());
        let state = WidgetState::default()
            .with_shape(Shape::RoundedRect)
            .hoverable()
            .with_size(80, 30);

        Self { id, state }
    }
}

impl Widget for Button {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn state(&mut self) -> &mut WidgetState {
        &mut self.state
    }
}
