use aplite_renderer::Scene;

use crate::context::Context;
use crate::view::IntoView;
use crate::widget::{InteractiveWidget, Mountable, Widget};

pub fn button<IV, F>(content: IV, f: F) -> impl IntoView
where
    IV: IntoView,
    F: Fn() + 'static,
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

impl<IV, F> Mountable for Button<IV, F>
where
    F: Fn() + 'static,
    IV: IntoView,
{
    fn build(self, cx: &mut Context) {
        self.content.build(cx);
    }
}

impl<IV: IntoView, F: Fn() + 'static> Widget for Button<IV, F> {
    fn layout(&self, cx: &mut Context) {
        self.content.layout(cx);
    }

    fn draw(&self, scene: &mut Scene) {
        self.content.draw(scene);
    }
}

impl<IV, F> InteractiveWidget for Button<IV, F>
where
    IV: IntoView,
    F: Fn() + 'static,
{
    fn execute(&self) {
        (self.f)()
    }
}

/*
#########################################################
#
# Text
#
#########################################################
*/

impl Widget for &'static str {
    fn layout(&self, cx: &mut Context) {}

    fn draw(&self, scene: &mut Scene) {}
}

impl Mountable for &'static str {
    fn build(self, cx: &mut Context) {}
}
