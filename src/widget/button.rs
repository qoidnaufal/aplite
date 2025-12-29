use aplite_renderer::Scene;
use aplite_types::Size;
use aplite_types::{CornerRadius, Rect, Rgba, theme::gruvbox_dark as theme};

use crate::context::Context;
use crate::layout::{AlignH, AlignV, LayoutCx, LayoutRules, Orientation, Padding, Spacing};
use crate::view::IntoView;
use crate::widget::{InteractiveWidget, Widget};

pub fn button<IV, F>(content: IV, f: F) -> Button<IV, F>
where
    IV: IntoView,
    F: Fn() + 'static,
{
    Button::new(content, f)
}

pub struct Button<IV: IntoView, F> {
    rect: Rect,
    background: Rgba,
    border_color: Rgba,
    border_width: f32,
    corner_radius: CornerRadius,
    content: IV::View,
    f: F
}

impl<IV: IntoView, F: Fn() + 'static> Button<IV, F> {
    fn new(content: IV, f: F) -> Self {
        Self {
            rect: Rect::new(0., 0., 100., 200.),
            background: theme::GREEN_0,
            border_color: theme::GREEN_1,
            border_width: 5.,
            corner_radius: CornerRadius::splat(5),
            content: content.into_view(),
            f,
        }
    }

    pub fn size(&mut self, size: Size) {
        self.rect.set_size(size);
    }

    pub fn background(&mut self, background: Rgba) {
        self.background = background;
    }

    pub fn border_color(&mut self, border_color: Rgba) {
        self.border_color = border_color;
    }

    pub fn border_width(&mut self, border_width: f32) {
        self.border_width = border_width;
    }

    pub fn corner_radius(&mut self, corner_radius: CornerRadius) {
        self.corner_radius = corner_radius;
    }
}

impl<IV: IntoView, F: Fn() + 'static> Widget for Button<IV, F> {
    fn layout_node_size(&self, _: Orientation) -> Size {
        self.rect.size()
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) {
        let pos = cx.get_next_pos(self.rect.size());
        self.rect.set_pos(pos);

        let rules = LayoutRules {
            padding: Padding::splat(5),
            orientation: crate::layout::Orientation::Horizontal,
            align_h: AlignH::Center,
            align_v: AlignV::Middle,
            spacing: Spacing(5),
        };
        let mut new_cx = LayoutCx::new(cx.cx, rules, self.rect, 0., 1);
        self.content.layout(&mut new_cx);
    }

    fn draw(&self, scene: &mut Scene) {
        self.content.draw(scene);
    }
}

impl<IV, F> IntoView for Button<IV, F>
where
    IV: IntoView,
    F: Fn() + 'static,
{
    type View = Self;

    fn into_view(self) -> Self {
        self
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
