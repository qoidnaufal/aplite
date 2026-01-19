use std::marker::PhantomData;
use aplite_renderer::Scene;
use aplite_types::{Color, CornerRadius, Length, Matrix3x2, PaintRef, Rect};
use aplite_types::theme::basic;

use crate::layout::{AlignH, AlignV, Axis, LayoutCx, LayoutRules, Padding, Spacing};
use crate::context::{BuildCx, Context};
use crate::state::BorderWidth;
use crate::view::IntoView;
use crate::widget::{Renderable, Widget};

pub fn hstack<C>(widget: C) -> Stack<C, Horizontal>
where
    C: IntoView,
{
    Stack::<C, Horizontal>::new(widget)
}

pub fn vstack<C>(widget: C) -> Stack<C, Vertical>
where
    C: IntoView,
{
    Stack::<C, Vertical>::new(widget)
}

pub trait StackDirection {
    const AXIS: Axis;
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
{
    pub(crate) content: C::View,
    style_fn: Option<Box<dyn Fn(&mut StackElement)>>,
    marker: PhantomData<AX>
}

impl<C, AX: StackDirection> Stack<C, AX>
where
    C: IntoView,
{
    fn new(widget: C) -> Self {
        Self {
            content: widget.into_view(),
            style_fn: None,
            marker: PhantomData,
        }
    }

    pub fn style(self, style_fn: impl Fn(&mut StackElement) + 'static) -> Self {
        Self {
            style_fn: Some(Box::new(style_fn)),
            ..self
        }
    }
}

impl<C, AX> Widget for Stack<C, AX>
where
    C: IntoView,
    AX: StackDirection + 'static,
{
    fn build(&self, cx: &mut BuildCx<'_>) {
        let mut state = StackElement {
            z_index: cx.get_z_index(),
            ..StackElement::new()
        };

        if let Some(style_fn) = self.style_fn.as_ref() {
            style_fn(&mut state);
        }

        cx.set_state(state);
        cx.with_id(0, |cx| self.content.build(cx));
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let state = cx.get_state::<StackElement>().unwrap();
        let bound = cx.bound;

        let width = match state.width {
            Length::Grow => bound.width,
            Length::Fixed(val) => val,
            Length::FitContent => 0.,
        };

        let height = match state.height {
            Length::Grow => bound.height,
            Length::Fixed(val) => val,
            Length::FitContent => 0.,
        };

        let rules = LayoutRules {
            padding: state.padding,
            axis: AX::AXIS,
            align_h: state.align_h,
            align_v: state.align_v,
            spacing: state.spacing,
        };

        let layout_node = Rect::new(bound.x, bound.y, width, height);

        match cx.rules.axis {
            Axis::Horizontal => {
                cx.bound.x += width + cx.rules.spacing.0 as f32;
            },
            Axis::Vertical =>  {
                cx.bound.y += height + cx.rules.spacing.0 as f32;
            },
        }

        cx.set_node(layout_node);

        let x = layout_node.x + rules.padding.left as f32;
        let y = layout_node.x + rules.padding.top as f32;
        let bound = Rect::new(x, y, width, height);

        let mut cx = LayoutCx::new(cx.cx, rules, bound);

        cx.with_id(0, |cx| self.content.layout(cx));
    }

    fn detect_hover(&self, cx: &mut Context) {
        let rect = cx.get_layout_node().unwrap();
        if rect.contains(&cx.cursor.hover.pos) {
            cx.with_id(0, |cx| self.content.detect_hover(cx))
        }
    }
}

// impl<C, AX> ForEachView for Stack<C, AX>
// where
//     C: IntoView,
//     C::View: ForEachView,
//     AX: StackDirection + 'static,
// {
//     fn for_each(&self, f: impl FnMut(&dyn Widget)) {
//         self.content.for_each(f);
//     }

//     fn for_each_mut(&mut self, f: impl FnMut(&mut dyn Widget)) {
//         self.content.for_each_mut(f);
//     }
// }

pub struct StackElement {
    pub width: Length,
    pub height: Length,
    pub background: Color,
    pub border_color: Color,
    pub border_width: BorderWidth,
    pub corner_radius: CornerRadius,
    pub padding: Padding,
    pub spacing: Spacing,
    pub align_h: AlignH,
    pub align_v: AlignV,
    z_index: u32,
}

impl StackElement {
    fn new() -> Self {
        Self {
            width: Length::Grow,
            height: Length::Grow,
            background: basic::TRANSPARENT,
            border_color: basic::TRANSPARENT,
            border_width: BorderWidth(0.),
            corner_radius: CornerRadius::splat(0),
            padding: Padding::splat(5),
            align_h: AlignH::Left,
            align_v: AlignV::Top,
            spacing: Spacing(5),
            z_index: 0,
        }
    }
}

impl Renderable for StackElement {
    fn render(&self, rect: &Rect, scene: &mut Scene) {
        scene.draw_rounded_rect(
            rect,
            &Matrix3x2::identity(),
            &PaintRef::from(&self.background),
            &PaintRef::from(&self.border_color),
            &self.border_width.0,
            &self.corner_radius,
        );
    }
}
