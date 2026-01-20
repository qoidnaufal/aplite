use aplite_renderer::Scene;
use aplite_types::{Length, Matrix3x2, PaintRef, Rect, rgb};
use aplite_types::{CornerRadius, Color};
use aplite_types::theme::gruvbox_dark as theme;

use crate::context::{BuildCx, CursorCx, LayoutCx};
use crate::layout::{AlignH, AlignV, Axis, LayoutRules, Padding, Spacing};
use crate::state::BorderWidth;
use crate::view::IntoView;
use crate::widget::{Renderable, Widget};

pub fn button<IV, F>(content: IV, f: F) -> Button<IV, F>
where
    IV: IntoView,
    F: Fn() + 'static,
{
    Button::new(content, f)
}

pub struct Button<IV: IntoView, F> {
    content: IV::View,
    callback: F,
    style_fn: Option<Box<dyn Fn(&mut ButtonElement)>>,
}

impl<IV: IntoView, F: Fn() + 'static> Button<IV, F> {
    fn new(content: IV, callback: F) -> Self {
        Self {
            content: content.into_view(),
            callback,
            style_fn: None,
        }
    }

    pub fn style(self, style_fn: impl Fn(&mut ButtonElement) + 'static) -> Self {
        Self {
            style_fn: Some(Box::new(style_fn)),
            ..self
        }
    }
}

impl<IV: IntoView, F: Fn() + 'static> Widget for Button<IV, F> {
    fn build(&self, cx: &mut BuildCx<'_>) {
        let z_index = cx.get_z_index();
        let mut state = ButtonElement {
            z_index,
            ..ButtonElement::new()
        };

        if let Some(style_fn) = self.style_fn.as_ref() {
            style_fn(&mut state);
        }

        cx.register_element(state);
        cx.with_id(0, |cx| self.content.build(cx));
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let state = cx.get_element::<ButtonElement>().unwrap();
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
            axis: state.axis,
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

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        let hovered = cx.get_layout_node()
            .map(|rect| rect.contains(cx.hover_pos()))
            .unwrap_or_default();

        if hovered {
            if !cx.with_id(0, |cx| self.content.detect_hover(cx)) {
                cx.set_id();
                cx.set_callback(Some(&self.callback));
            }
        }

        hovered
    }
}

impl<IV: IntoView, F: Fn() + 'static> IntoView for Button<IV, F> {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ButtonElement {
    pub width: Length,
    pub height: Length,
    pub padding: Padding,
    pub spacing: Spacing,
    pub align_h: AlignH,
    pub align_v: AlignV,
    pub background: Color,
    pub border_color: Color,
    pub border_width: BorderWidth,
    pub corner_radius: CornerRadius,
    axis: Axis,
    z_index: u32,
}

impl std::fmt::Debug for ButtonElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ButtonElement")
            .finish_non_exhaustive()
    }
}

impl ButtonElement {
    fn new() -> Self {
        Self {
            width: Length::Grow,
            height: Length::Grow,
            padding: Padding::splat(5),
            spacing: Spacing(5),
            align_h: AlignH::Center,
            align_v: AlignV::Middle,
            axis: Axis::Horizontal,
            background: theme::GREEN_0,
            border_color: theme::GREEN_1,
            border_width: BorderWidth(5.),
            corner_radius: CornerRadius::splat(5),
            z_index: 0,
        }
    }
}

impl Renderable for ButtonElement {
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
