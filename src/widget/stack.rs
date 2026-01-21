use std::marker::PhantomData;
use aplite_renderer::Scene;
use aplite_types::{Color, CornerRadius, Length, Matrix3x2, PaintRef, Rect};
use aplite_types::theme::basic;

use crate::layout::{AlignH, AlignV, Axis, LayoutRules, Padding, Spacing};
use crate::context::{CursorCx, BuildCx, LayoutCx};
use crate::state::BorderWidth;
use crate::view::IntoView;
use crate::widget::{Renderable, Widget};

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

pub trait StackDirection: 'static {
    const AXIS: Axis;
}

pub struct Horizontal; impl StackDirection for Horizontal {
    const AXIS: Axis = Axis::Horizontal;
}

pub struct Vertical; impl StackDirection for Vertical {
    const AXIS: Axis = Axis::Vertical;
}

pub struct Stack<IV, AX>
where
    IV: IntoView,
{
    pub(crate) content: IV::View,
    style_fn: Option<Box<dyn Fn(&mut StackElement)>>,
    marker: PhantomData<AX>
}

impl<IV, AX: StackDirection> Stack<IV, AX>
where
    IV: IntoView,
{
    fn new(widget: IV) -> Self {
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

impl<IV, AX> Widget for Stack<IV, AX>
where
    IV: IntoView,
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

        cx.register_element(state);
        cx.with_id(0, |cx| self.content.build(cx))
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let state = cx.get_element::<StackElement>().unwrap();

        let width = match state.width {
            Length::Grow => cx.bound.width,
            Length::Fixed(val) => val,
            Length::FitContent => 0.,
        };

        let height = match state.height {
            Length::Grow => cx.bound.height,
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

        let layout_node = Rect::new(
            cx.bound.x,
            cx.bound.y,
            width,
            height,
        );

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

        let bound = Rect::new(
            x,
            y,
            width - rules.padding.horizontal() as f32,
            height - rules.padding.vertical() as f32
        );

        let mut cx = LayoutCx::derive(cx, rules, bound);

        cx.with_id(0, |cx| self.content.layout(cx));
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        let hovered = cx.get_layout_node()
            .map(|rect| rect.contains(cx.hover_pos()))
            .unwrap_or_default();

        if hovered {
            if !cx.with_id(0, |cx| self.content.detect_hover(cx)) {
                cx.set_id();
            }
        }

        hovered
    }
}

impl<IV: IntoView, AX: StackDirection> IntoView for Stack<IV, AX> {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

#[derive(PartialEq, Eq)]
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

impl std::fmt::Debug for StackElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StackElement")
            .finish_non_exhaustive()
    }
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
            padding: Padding::splat(0),
            align_h: AlignH::Left,
            align_v: AlignV::Top,
            spacing: Spacing(0),
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
