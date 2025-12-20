use aplite_renderer::Scene;
use aplite_types::{Rgba, Size, Unit};
use aplite_storage::Entity;

use crate::layout::*;
use crate::view::{View, IntoView};
use crate::context::Context;
use crate::callback::WidgetEvent;

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
    fn build(self, cx: &mut Context) -> Entity;

    fn layout(&mut self, cx: &mut Context);

    fn draw(&self, scene: &mut Scene);
}

impl<F, IV> Widget for F
where
    F: FnOnce() -> IV + 'static,
    IV: IntoView,
{
    fn build(self, storage: &mut Context) -> Entity {
        self().into_view().build(storage)
    }

    fn layout(&mut self, _: &mut Context) {}

    fn draw(&self, _: &mut aplite_renderer::Scene) {}
}

// -------------------------------------

pub(crate) fn window(size: Size) -> WindowWidget {
    WindowWidget::new(size)
}

pub(crate) struct WindowWidget {
    size: Size,
    layout_rules: LayoutRules,
}

impl WindowWidget {
    pub(crate) fn new(size: Size) -> Self {
        let layout_rules = LayoutRules::default();
        Self {
            size,
            layout_rules,
        }
    }
}

impl Widget for WindowWidget {
    fn build(self, cx: &mut Context) -> Entity {
        cx.mount(self)
    }

    fn layout(&mut self, cx: &mut Context) {
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

// -------------------------------------

pub fn circle() -> impl IntoView {
    CircleWidget::new()
}

struct CircleWidget {
    radius: Unit,
}

impl CircleWidget {
    fn new() -> Self {
        Self {
            radius: Unit::Fixed(100.),
        }
    }

    fn radius(self, radius: Unit) -> Self {
        Self {
            radius,
            ..self
        }
    }
}

impl Widget for CircleWidget {
    fn build(self, cx: &mut Context) -> Entity {
        cx.mount(self)
    }

    fn layout(&mut self, cx: &mut Context) {
       todo!() 
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}
