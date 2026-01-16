use std::marker::PhantomData;
use aplite_types::{Length, Color};
use aplite_types::theme::basic;

use crate::layout::{AlignH, AlignV, LayoutCx, Axis, Padding, Spacing};
use crate::context::BuildCx;
use crate::state::BorderWidth;
use crate::view::IntoView;
use crate::widget::Widget;

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
    style_fn: Option<Box<dyn Fn(&mut StackState)>>,
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

    pub fn style(self, style_fn: impl Fn(&mut StackState) + 'static) -> Self {
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
        let mut state = StackState::new();
        if let Some(style_fn) = self.style_fn.as_ref() {
            style_fn(&mut state);
        }

        cx.set_state(StackState::new());
        cx.with_id(0, |cx| self.content.build(cx));
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let _available_space = cx.get_available_space();
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

pub struct StackState {
    pub width: Length,
    pub height: Length,
    pub background: Color,
    pub border_color: Color,
    pub border_width: BorderWidth,
    pub padding: Padding,
    pub spacing: Spacing,
    pub align_h: AlignH,
    pub align_v: AlignV,
}

impl StackState {
    fn new() -> Self {
        Self {
            width: Length::Grow,
            height: Length::Grow,
            background: basic::TRANSPARENT,
            border_color: basic::TRANSPARENT,
            border_width: BorderWidth(0.),
            padding: Padding::splat(5),
            align_h: AlignH::Left,
            align_v: AlignV::Top,
            spacing: Spacing(5),
        }
    }
}
