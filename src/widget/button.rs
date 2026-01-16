use aplite_types::Length;
use aplite_types::{CornerRadius, Color};
use aplite_types::theme::gruvbox_dark as theme;

use crate::context::BuildCx;
use crate::layout::{AlignH, AlignV, LayoutCx, Axis, Padding, Spacing};
use crate::state::BorderWidth;
use crate::view::{ForEachView, IntoView};
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
    f: F,
    style_fn: Option<Box<dyn Fn(&mut ButtonState)>>,
}

impl<IV: IntoView, F: Fn() + 'static> Button<IV, F> {
    fn new(content: IV, f: F) -> Self {
        Self {
            content: content.into_view(),
            f,
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
        cx.insert_state(state);
        cx.with_id(0, |cx| self.content.build(cx));
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {}
}

impl<IV, F> ForEachView for Button<IV, F>
where
    IV: IntoView,
    IV::View: ForEachView,
    F: Fn() + 'static, {}

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
    content_layout: Axis,
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
            content_layout: Axis::Horizontal,
            background: theme::GREEN_0,
            border_color: theme::GREEN_1,
            border_width: BorderWidth(5.),
            corner_radius: CornerRadius::splat(5),
            z_index: 0,
        }
    }
}
