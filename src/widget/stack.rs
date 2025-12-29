use std::marker::PhantomData;
use aplite_renderer::{Scene, DrawArgs};
use aplite_types::{Length, Matrix3x2, PaintRef, Rect, Rgba, Size};
use aplite_types::theme::basic;

use crate::layout::{AlignH, AlignV, LayoutCx, LayoutRules, Orientation, Padding, Spacing};
use crate::context::Context;
use crate::state::{Background, BorderColor, BorderWidth};
use crate::view::IntoView;
use crate::widget::Widget;

pub fn hstack<IV>(widget: IV) -> Stack<IV, Horizontal>
where
    IV: IntoView,
{
    Stack::<IV, Horizontal>::new(widget)
}

pub fn vstack<IV>(widget: IV) -> Stack<IV, Vertical>
where
    IV: IntoView,
{
    Stack::<IV, Vertical>::new(widget)
}

pub trait StackOrientation {
    const ORIENTATION: Orientation;
}

pub struct Horizontal; impl StackOrientation for Horizontal {
    const ORIENTATION: Orientation = Orientation::Horizontal;
}

pub struct Vertical; impl StackOrientation for Vertical {
    const ORIENTATION: Orientation = Orientation::Vertical;
}

pub struct Stack<IV, SO> {
    content: IV,
    rect: Rect,
    background: Background,
    border_color: BorderColor,
    border_width: BorderWidth,
    padding: Padding,
    spacing: Spacing,
    align_h: AlignH,
    align_v: AlignV,
    marker: PhantomData<SO>
}

impl<IV, SO: StackOrientation> Stack<IV, SO>
where
    IV: IntoView,
{
    fn new(widget: IV) -> Self {
        Self {
            content: widget,
            rect: Rect::default(),
            background: Background(basic::TRANSPARENT),
            border_color: BorderColor(basic::TRANSPARENT),
            border_width: BorderWidth(0.),
            padding: Padding::splat(5),
            align_h: AlignH::Left,
            align_v: AlignV::Top,
            spacing: Spacing(5),
            marker: PhantomData,
        }
    }

    pub fn width(self, length: Length) -> Self {
        let _ = length;
        self
    }

    pub fn height(self, length: Length) -> Self {
        let _ = length;
        self
    }

    pub fn padding(self, padding: Padding) -> Self {
        Self { padding, ..self }
    }

    pub fn spacing(self, spacing: u8) -> Self {
        Self { spacing: Spacing(spacing), ..self }
    }

    pub fn align_h(self, align_h: AlignH) -> Self {
        Self { align_h, ..self }
    }

    pub fn align_v(self, align_v: AlignV) -> Self {
        Self { align_v, ..self }
    }

    pub fn background(self, color: Rgba) -> Self {
        Self { background: Background(color), ..self }
    }

    pub fn border_color(self, color: Rgba) -> Self {
        Self { border_color: BorderColor(color), ..self }
    }

    pub fn border_width(self, width: f32) -> Self {
        Self { border_width: BorderWidth(width), ..self }
    }
}

impl<IV, SO> Widget for Stack<IV, SO>
where
    IV: IntoView,
    SO: StackOrientation + 'static,
{
    fn layout_node_size(&self, _: Orientation) -> Size {
        let mut s = Size::default();
        let content_size = self.content.layout_node_size(SO::ORIENTATION);

        s.width = content_size.width + self.padding.horizontal() as f32;
        s.height = content_size.height + self.padding.vertical() as f32;

        s
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) {
        let pos = cx.get_next_pos(self.rect.size());
        self.rect.set_pos(pos);

        let rules = LayoutRules {
            padding: self.padding,
            orientation: SO::ORIENTATION,
            align_h: self.align_h,
            align_v: self.align_v,
            spacing: self.spacing,
        };
        let mut cx = LayoutCx::new(cx.cx, rules, self.rect, 0., 0);

        self.content.layout(&mut cx);
    }

    fn draw(&self, scene: &mut Scene) {
        scene.draw(DrawArgs {
            rect: &self.rect,
            transform: &Matrix3x2::identity(),
            background_paint: &PaintRef::Color(&self.background.0),
            border_paint: &PaintRef::Color(&self.border_color.0),
            border_width: &self.border_width.0,
            shape: &aplite_renderer::Shape::Rect,
            corner_radius: &aplite_types::CornerRadius::splat(0),
        });
        self.content.draw(scene);
    }
}

impl<IV, SO> IntoView for Stack<IV, SO>
where
    IV: IntoView,
    SO: StackOrientation + 'static,
{
    type View = Stack<<IV as IntoView>::View, SO>;

    fn into_view(self) -> Self::View {
        Stack {
            content: self.content.into_view(),
            rect: self.rect,
            background: self.background,
            border_color: self.border_color,
            border_width: self.border_width,
            padding: self.padding,
            spacing: self.spacing,
            align_h: self.align_h,
            align_v: self.align_v,
            marker: PhantomData,
        }
    }
}
