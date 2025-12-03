use aplite_types::{Rgba, Unit};
use aplite_types::theme::basic;
use aplite_renderer::Scene;

use crate::context::Context;
use super::Widget;

pub fn button() -> Button { Button::new() }

pub struct Button {
    width: Unit,
    height: Unit,
    background: Rgba,
    border_color: Rgba,
    border_width: f32,
}

impl Button {
    pub fn new() -> Self {
        Self {
            width: Unit::Fixed(80.),
            height: Unit::Fixed(80.),
            background: basic::RED,
            border_color: basic::RED,
            border_width: 0.0,
        }
    }
}

impl Widget for Button {
    fn build(self, cx: &mut Context) -> aplite_storage::Entity {
        todo!()
    }

    fn layout(&mut self, cx: &mut Context) {
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

// impl InteractiveWidget for Button {}
