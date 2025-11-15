use aplite_types::{Rgba, Unit};
use aplite_types::theme::basic;

use crate::view::IntoView;
use super::{Widget, InteractiveWidget};

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

impl IntoView for Button {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

impl Widget for Button {}

impl InteractiveWidget for Button {}
