use std::marker::PhantomData;
use aplite_renderer::{Scene, DrawArgs};
use aplite_types::{Length, Matrix3x2, PaintRef, Rect, Rgba, Size};
use aplite_types::theme::basic;

use crate::layout::{AlignH, AlignV, LayoutCx, LayoutRules, Axis, Padding, Spacing};
use crate::context::Context;
use crate::state::{Background, BorderColor, BorderWidth};
use crate::view::{ForEachView, IntoView};
use crate::widget::Widget;

pub fn hstack<C>(widget: C) -> Stack<C, Horizontal>
where
    C: IntoView,
    C::View: ForEachView,
{
    Stack::<C, Horizontal>::new(widget)
}

pub fn vstack<C>(widget: C) -> Stack<C, Vertical>
where
    C: IntoView,
    C::View: ForEachView,
{
    Stack::<C, Vertical>::new(widget)
}

pub trait StackDirection {
    const AXIS: Axis;

    // fn layout() {
    //     match Self::AXIS {
    //         Axis::Horizontal => todo!(),
    //         Axis::Vertical => todo!(),
    //     }
    // }
}

pub struct Horizontal; impl StackDirection for Horizontal {
    const AXIS: Axis = Axis::Horizontal;
}

pub struct Vertical; impl StackDirection for Vertical {
    const AXIS: Axis = Axis::Vertical;
}

pub struct Stack<C, AX>
where
    C: IntoView,
    C::View: ForEachView,
{
    pub(crate) content: C::View,
    width: Length,
    height: Length,
    background: Background,
    border_color: BorderColor,
    border_width: BorderWidth,
    padding: Padding,
    spacing: Spacing,
    align_h: AlignH,
    align_v: AlignV,
    marker: PhantomData<AX>
}

impl<C, AX: StackDirection> Stack<C, AX>
where
    C: IntoView,
    C::View: ForEachView,
{
    fn new(widget: C) -> Self {
        Self {
            content: widget.into_view(),
            width: Length::Grow,
            height: Length::Grow,
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

    pub fn with_width(self, width: Length) -> Self {
        Self { width, ..self }
    }

    pub fn with_height(self, height: Length) -> Self {
        Self { height, ..self }
    }

    pub fn with_padding(self, padding: Padding) -> Self {
        Self { padding, ..self }
    }

    pub fn with_spacing(self, spacing: u8) -> Self {
        Self { spacing: Spacing(spacing), ..self }
    }

    pub fn with_align_h(self, align_h: AlignH) -> Self {
        Self { align_h, ..self }
    }

    pub fn with_align_v(self, align_v: AlignV) -> Self {
        Self { align_v, ..self }
    }

    pub fn with_background(self, color: Rgba) -> Self {
        Self { background: Background(color), ..self }
    }

    pub fn with_border_color(self, color: Rgba) -> Self {
        Self { border_color: BorderColor(color), ..self }
    }

    pub fn with_border_width(self, width: f32) -> Self {
        Self { border_width: BorderWidth(width), ..self }
    }
}

impl<C, AX> Widget for Stack<C, AX>
where
    C: IntoView,
    C::View: ForEachView,
    AX: StackDirection + 'static,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout_node_size(&self, bound: Size) -> Size {
        let mut content_size = Size::default();
        let child_count = self.content.count();

        match AX::AXIS {
            Axis::Horizontal => {
                let bound = Size::new(bound.width / child_count as f32, bound.height);

                self.content.for_each(|child| {
                    let cs = child.layout_node_size(bound);
                    content_size.width += cs.width;
                    content_size.height = content_size.height.max(cs.height);
                });

                content_size.width += ((child_count - 1) * self.spacing.0 as usize) as f32;
            },
            Axis::Vertical => {
                let bound = Size::new(bound.width, bound.height / child_count as f32);

                self.content.for_each(|w| {
                    let cs = w.layout_node_size(bound);
                    content_size.height += cs.height;
                    content_size.width = content_size.width.max(cs.width);
                });

                content_size.height += ((child_count - 1) * self.spacing.0 as usize) as f32;
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
            orientation: AX::AXIS,
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
            background_paint: &PaintRef::Color(&self.background.0),
            border_paint: &PaintRef::Color(&self.border_color.0),
            border_width: &self.border_width.0,
            shape: &aplite_renderer::Shape::Rect,
            corner_radius: &aplite_types::CornerRadius::splat(0),
        });
        self.content.draw(scene);
    }
}

impl<C, AX> ForEachView for Stack<C, AX>
where
    C: IntoView,
    C::View: ForEachView,
    AX: StackDirection + 'static,
{
    fn for_each(&self, f: impl FnMut(&dyn Widget)) {
        self.content.for_each(f);
    }

    fn for_each_mut(&mut self, f: impl FnMut(&mut dyn Widget)) {
        self.content.for_each_mut(f);
    }
}

impl<C, AX> IntoView for Stack<C, AX>
where
    C: IntoView,
    C::View: ForEachView,
    AX: StackDirection + 'static,
{
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}
