use aplite_renderer::Scene;
use aplite_types::{Rgba, Size, Unit};
use aplite_storage::{
    ArenaItem,
    Entity,
};

use crate::layout::*;
use crate::view::IntoView;
use crate::context::Context;

mod button;
mod image;
mod stack;

pub use {
    button::*,
    image::*,
    stack::*,
};


/// main building block to create a renderable component
pub trait Widget {
    fn layout(&mut self, cx: &mut Context) {}

    fn draw(&self, scene: &mut Scene) {}
}

impl Widget for ArenaItem<dyn Widget> {
    fn layout(&mut self, cx: &mut Context) {
        self.as_mut().layout(cx);
    }

    fn draw(&self, scene: &mut Scene) {
        self.as_ref().draw(scene);
    }
}

impl std::fmt::Debug for dyn Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .finish()
    }
}

// -------------------------------------

pub trait ParentWidget: Widget {}

pub trait InteractiveWidget: Widget {}

// -------------------------------------

pub(crate) fn window(size: Size) -> WindowWidget {
    WindowWidget::new(size)
}

pub(crate) struct WindowWidget {
    id: Entity,
    size: Size,
    layout_rules: LayoutRules,
}

impl WindowWidget {
    pub(crate) fn new(size: Size) -> Self {
        let layout_rules = LayoutRules::default();
        Self {
            id: Entity::new(0, 0),
            size,
            layout_rules,
        }
    }
}

impl Widget for WindowWidget {}

impl IntoView for WindowWidget {
    type View = Self;
    fn into_view(self) -> Self::View {
        self
    }
}

// -------------------------------------

pub fn circle() -> CircleWidget {
    CircleWidget::new()
}

pub struct CircleWidget {
    radius: Unit,
    background: Rgba,
    border_color: Rgba,
    border_width: f32,
}

impl CircleWidget {
    pub fn new() -> Self {
        Self {
            radius: Unit::Fixed(100.),
            background: Rgba::RED,
            border_color: Rgba::RED,
            border_width: 0.0,
        }
    }

    pub fn radius(self, radius: Unit) -> Self {
        Self {
            radius,
            ..self
        }
    }

    pub fn background(self, color: Rgba) -> Self {
        Self {
            background: color,
            ..self
        }
    }
}

impl Widget for CircleWidget {}

impl IntoView for CircleWidget {
    type View = Self;
    fn into_view(self) -> Self::View {
        self
    }
}
