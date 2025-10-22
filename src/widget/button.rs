use aplite_renderer::{Shape, Scene};

use super::{Widget, WidgetId};
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

        Self {
            state
        }
    }
}

impl Widget for Button {
    fn state_ref(&self) -> &WidgetState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn draw(&self, scene: &mut Scene) {
        if self.state.flag.visible {
            scene.draw_rounded_rect(
                &self.state.rect,
                &self.state.transform,
                &self.state.background.paint.as_paint_ref(),
                &self.state.border.paint.as_paint_ref(),
                &self.state.border.width,
                &self.state.corner_radius,
            );
        }
    }
}
