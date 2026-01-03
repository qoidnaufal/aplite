use aplite_renderer::{DrawArgs, Scene};
use aplite_types::{Length, Matrix3x2, PaintRef, Size};
use aplite_types::{CornerRadius, Rect, Rgba, theme::gruvbox_dark as theme};

use crate::context::Context;
use crate::layout::{AlignH, AlignV, LayoutCx, LayoutRules, Axis, Padding, Spacing};
use crate::view::{ForEachView, IntoView};
use crate::widget::{InteractiveWidget, Widget};

pub fn button<IV, F>(content: IV, f: F) -> Button<IV, F>
where
    IV: IntoView,
    F: Fn() + 'static,
{
    Button::new(content, f)
}

pub struct Button<IV: IntoView, F> {
    width: Length,
    height: Length,
    padding: Padding,
    spacing: Spacing,
    align_h: AlignH,
    align_v: AlignV,
    content_layout: Axis,
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
            width: Length::MinContent(100.),
            height: Length::MinContent(100.),
            padding: Padding::splat(5),
            spacing: Spacing(5),
            align_h: AlignH::Center,
            align_v: AlignV::Middle,
            content_layout: Axis::Horizontal,
            background: theme::GREEN_0,
            border_color: theme::GREEN_1,
            border_width: 5.,
            corner_radius: CornerRadius::splat(5),
            content: content.into_view(),
            f,
        }
    }

    pub fn with_width(self, width: Length) -> Self {
        Self{ width, ..self }
    }

    pub fn with_height(self, height: Length) -> Self {
        Self{ height, ..self }
    }

    pub fn with_background(self, background: Rgba) -> Self {
        Self { background, ..self }
    }

    pub fn with_border_color(self, border_color: Rgba) -> Self {
        Self { border_color, ..self }
    }

    pub fn with_border_width(self, border_width: f32) -> Self {
        Self { border_width, ..self }
    }

    pub fn with_corner_radius(self, corner_radius: CornerRadius) -> Self {
        Self { corner_radius, ..self }
    }

    pub fn with_content_layout(self, content_layout: Axis) -> Self {
        Self { content_layout, ..self }
    }
}

impl<IV: IntoView, F: Fn() + 'static> Widget for Button<IV, F> {
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout_node_size(&self, bound: Size) -> Size {
        let mut content_size = Size::default();

        match self.content_layout {
            Axis::Horizontal => {
                let _c_width = self.content.width();
            },
            Axis::Vertical => {
                let _c_heigth = self.content.height();
            }
        }

        content_size.width += self.padding.horizontal() as f32;
        content_size.height += self.padding.vertical() as f32;

        content_size
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let size = Size::default();
        let pos = cx.get_next_pos(size);
        let rect = Rect::from_vec2f_size(pos, size);

        let rules = LayoutRules {
            padding: self.padding,
            orientation: Axis::Horizontal,
            align_h: self.align_h,
            align_v: self.align_v,
            spacing: self.spacing,
        };

        let mut cx = LayoutCx::new(cx.cx, rules, rect, 0., 0);

        self.content.layout(&mut cx);
    }

    fn draw(&self, scene: &mut Scene) {
        scene.draw(DrawArgs {
            rect: &Rect::default(),
            transform: &Matrix3x2::identity(),
            background_paint: &PaintRef::Color(&self.background),
            border_paint: &PaintRef::Color(&self.border_color),
            border_width: &self.border_width,
            shape: &aplite_renderer::Shape::RoundedRect,
            corner_radius: &self.corner_radius,
        });
        self.content.draw(scene);
    }
}

impl<IV, F> ForEachView for Button<IV, F>
where
    IV: IntoView,
    IV::View: ForEachView,
    F: Fn() + 'static, {}

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
    fn trigger(&self) {
        (self.f)()
    }
}
