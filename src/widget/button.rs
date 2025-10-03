use aplite_renderer::Shape;
use super::Widget;
use crate::state::WidgetState;

pub fn button() -> Button { Button::new() }

pub struct Button {
    state: WidgetState,
}

impl Button {
    pub fn new() -> Self {
        let state = WidgetState::default()
            .with_shape(Shape::RoundedRect)
            .hoverable()
            .with_size(80, 30);

        Self { state }
    }
}

impl Widget for Button {
    fn state(&self) -> &WidgetState {
        &self.state
    }
}
