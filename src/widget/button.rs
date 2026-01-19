use aplite_types::{Length, Rect};
use aplite_types::{CornerRadius, Color};
use aplite_types::theme::gruvbox_dark as theme;

use crate::context::BuildCx;
use crate::layout::{AlignH, AlignV, Axis, LayoutCx, LayoutRules, Padding, Spacing};
use crate::state::BorderWidth;
use crate::view::IntoView;
use crate::widget::Widget;

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
    style_fn: Option<Box<dyn Fn(&mut ButtonState)>>,
}

impl<IV: IntoView, F: Fn() + 'static> Button<IV, F> {
    fn new(content: IV, callback: F) -> Self {
        Self {
            content: content.into_view(),
            callback,
            style_fn: None,
        }
    }

    pub fn style(self, style_fn: impl Fn(&mut ButtonState) + 'static) -> Self {
        Self {
            style_fn: Some(Box::new(style_fn)),
            ..self
        }
    }
}

impl<IV: IntoView, F: Fn() + 'static> Widget for Button<IV, F> {
    fn build(&self, cx: &mut BuildCx<'_>) {
        let z_index = cx.get_z_index();
        let mut state = ButtonState {
            z_index,
            ..ButtonState::new()
        };

        if let Some(style_fn) = self.style_fn.as_ref() {
            style_fn(&mut state);
        }

        cx.set_state(state);
        cx.with_id(0, |cx| self.content.build(cx));
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let id = cx.get_id().copied().unwrap();
        let any = &cx.cx.states[id.0 as usize];
        let state = any.downcast_ref::<ButtonState>().unwrap();
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonState {
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

impl ButtonState {
    fn new() -> Self {
        Self {
            width: Length::FitContent,
            height: Length::FitContent,
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
