use aplite_types::{Length, Size};
use aplite_types::{CornerRadius, Rect, Color};
use aplite_types::theme::gruvbox_dark as theme;
use aplite_reactive::*;

use crate::context::Context;
use crate::layout::{AlignH, AlignV, LayoutCx, Axis, Padding, Spacing};
use crate::state::BorderWidth;
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
    content: IV::View,
    f: F,
    state: Signal<ButtonState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonState {
    width: Length,
    height: Length,
    padding: Padding,
    spacing: Spacing,
    align_h: AlignH,
    align_v: AlignV,
    content_layout: Axis,
    background: Color,
    border_color: Color,
    border_width: BorderWidth,
    corner_radius: CornerRadius,
}

impl<IV: IntoView, F: Fn() + 'static> Button<IV, F> {
    fn new(content: IV, f: F) -> Self {
        Self {
            content: content.into_view(),
            f,
            state: Signal::new(ButtonState::new()),
        }
    }

    pub fn with_corner_radius(self, corner_radius: CornerRadius) -> Self {
        self.state.update_untracked(|state| state.corner_radius = corner_radius);
        self
    }

    pub fn with_width(self, width: Length) -> Self {
        self.state.update_untracked(|state| state.width = width);
        self
    }

    pub fn with_height(self, height: Length) -> Self {
        self.state.update_untracked(|state| state.height = height);
        self
    }

    pub fn with_background(self, color: Color) -> Self {
        self.state.update_untracked(|state| state.background = color);
        self
    }
}

impl<IV: IntoView, F: Fn() + 'static> Widget for Button<IV, F> {
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        // let size = Size::default();
        // let pos = cx.get_next_pos(size);
        // let rect = Rect::from_vec2f_size(pos, size);

        // let rules = LayoutRules {
        //     padding: self.padding,
        //     orientation: Axis::Horizontal,
        //     align_h: self.align_h,
        //     align_v: self.align_v,
        //     spacing: self.spacing,
        // };

        // let mut cx = LayoutCx::new(cx.cx, rules, rect, 0., 0);

        // self.content.layout(&mut cx);
    }
}

impl<IV, F> ForEachView for Button<IV, F>
where
    IV: IntoView,
    IV::View: ForEachView,
    F: Fn() + 'static, {}

impl<IV, F> InteractiveWidget for Button<IV, F>
where
    IV: IntoView,
    F: Fn() + 'static,
{
    fn trigger(&self) {
        (self.f)()
    }
}

impl ButtonState {
    fn new() -> Self {
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
            border_width: BorderWidth(5.),
            corner_radius: CornerRadius::splat(5),
        }
    }
}
