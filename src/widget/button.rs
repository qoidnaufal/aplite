use aplite_types::{Rgba, Unit};
use aplite_types::theme::basic;
use aplite_renderer::Scene;
use aplite_storage::Entity;

use crate::context::Context;
use crate::view::{IntoView, View};
use crate::widget::Widget;
// use crate::callback::InteractiveWidget;

pub fn button<IV, F>(content: IV, f: F) -> impl IntoView
where
    IV: IntoView,
    F: FnMut() + 'static,
{
    Button::new(content, f)
}

struct Button<IV, F> {
    content: IV,
    f: F
}

impl<IV, F> Button<IV, F> {
    fn new(content: IV, f: F) -> Self {
        Self {
            content,
            f,
        }
    }
}

impl<IV: IntoView, F: FnMut() + 'static> Widget for Button<IV, F> {
    fn build(self, cx: &mut Context) -> Entity {
        let entity = cx.mount(self);
        entity
    }

    fn layout(&mut self, cx: &mut Context) {
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

// impl<IV: IntoView, F: FnMut() + 'static> InteractiveWidget for Button<IV, F> {}
