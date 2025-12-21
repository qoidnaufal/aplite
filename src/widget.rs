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

/*
#########################################################
#
# Widget Trait
#
#########################################################
*/

/// main building block to create a renderable component
pub trait Widget {
    fn layout(&mut self, cx: &mut Context);

    fn draw(&self, scene: &mut Scene);
}

/*
#########################################################
#
# ViewFn
#
#########################################################
*/

impl<F, IV> Widget for F
where
    F: FnOnce() -> IV + 'static,
    IV: IntoView,
{
    fn layout(&mut self, _: &mut Context) {}

    fn draw(&self, _: &mut aplite_renderer::Scene) {}
}

/*
#########################################################
#
# WindowWidget
#
#########################################################
*/

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
    fn layout(&mut self, cx: &mut Context) {
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

/*
#########################################################
#
# Circle
#
#########################################################
*/

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
    fn layout(&mut self, cx: &mut Context) {
       todo!() 
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}
