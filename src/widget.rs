use std::sync::Arc;

use aplite_reactive::*;
use aplite_types::{Color, Length, Rect, rgb, rgba};

use crate::{layout::Axis, state::BorderWidth};
use crate::layout::LayoutCx;
use crate::view::IntoView;
use crate::context::BuildCx;

mod button;
mod image;
mod stack;

pub use {
    button::*,
    image::*,
    stack::*,
};

/*
#########################################################
#
# Widget Trait
#
#########################################################
*/

/// main building block to create a renderable component
pub trait Widget {
    fn debug_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn build(&self, cx: &mut BuildCx<'_>);

    fn layout(&self, cx: &mut LayoutCx<'_>);
}

/*
#########################################################
#
# ViewFn
#
#########################################################
*/

impl<F, IV> Widget for F
where
    F: Fn() -> IV + 'static,
    IV: IntoView,
{
    fn debug_name(&self) -> &'static str {
        self().debug_name()
    }

    fn build(&self, cx: &mut BuildCx<'_>) {
        self().build(cx)
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self().layout(cx);
    }
}

/*
#########################################################
#
# Circle
#
#########################################################
*/

pub fn circle() -> CircleWidget {
    CircleWidget {
        style_fn: None,
    }
}

pub struct CircleWidget {
    style_fn: Option<Box<dyn Fn(&mut CircleState)>>,
}

impl CircleWidget {
    pub fn style(self, style_fn: impl Fn(&mut CircleState) + 'static) -> Self {
        Self {
            style_fn: Some(Box::new(style_fn)),
            ..self
        }
    }
}

pub struct CircleState {
    pub radius: Length,
    pub rect: Rect,
    pub background: Color,
    pub border_color: Color,
    pub border_width: BorderWidth,
}

impl CircleState {
    fn new() -> Self {
        Self {
            radius: Length::Grow,
            rect: Rect::default(),
            background: rgba(0xff6969ff),
            border_color: rgb(0x000000),
            border_width: BorderWidth(0.),
        }
    }
}

impl Widget for CircleWidget {
    fn build(&self, cx: &mut BuildCx<'_>) {
        let mut state = CircleState::new();
        if let Some(style_fn) = self.style_fn.as_ref() {
            style_fn(&mut state);
        }

        cx.set_state(state);
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let id = cx.get_id().copied().unwrap();
        let any = &cx.cx.states[id.0 as usize];
        let state = any.downcast_ref::<CircleState>().unwrap();
        let bound = cx.bound;

        let radius = match state.radius {
            Length::Grow => bound.width.max(bound.height),
            Length::Fixed(val) => val,
            Length::FitContent => 0.,
        };

        let layout_node = Rect::new(bound.x, bound.y, radius, radius);

        match cx.rules.axis {
            Axis::Horizontal => {
                cx.bound.x += radius + cx.rules.spacing.0 as f32;
            },
            Axis::Vertical =>  {
                cx.bound.y += radius + cx.rules.spacing.0 as f32;
            },
        }

        cx.set_node(layout_node);
    }
}

/*
#########################################################
#
# Either
#
#########################################################
*/

enum Either<VT, VF> {
    True(VT),
    False(VF)
}

impl<VT, VF> Widget for Either<VT, VF>
where
    VT: IntoView,
    VF: IntoView,
{
    fn debug_name(&self) -> &'static str {
        match self {
            Either::True(t) => t.debug_name(),
            Either::False(f) => f.debug_name(),
        }
    }

    fn build(&self, cx: &mut BuildCx<'_>) {
        match self {
            Either::True(t) => t.build(cx),
            Either::False(f) => f.build(cx),
        }
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        match self {
            Either::True(t) => t.layout(cx),
            Either::False(f) => f.layout(cx),
        }
    }
}

pub fn either<W, TrueFn, FalseFn, VT, VF>(
    when: W,
    content_true: TrueFn,
    content_false: FalseFn,
) -> impl IntoView
where
    W: Fn() -> bool + 'static,
    TrueFn: Fn() -> VT + 'static,
    FalseFn: Fn() -> VF + 'static,
    VT: IntoView,
    VF: IntoView,
{
    let when = Memo::new(move |_| when());

    move || match when.get() {
        true => Either::True(content_true().into_view()),
        false => Either::False(content_false().into_view()),
    }
}

/*
#########################################################
#
# Vec<W>
#
#########################################################
*/

impl<IV: IntoView> Widget for Vec<IV> {
    fn build(&self, cx: &mut BuildCx<'_>) {
        self.iter().for_each(|widget| widget.build(cx));
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.iter().for_each(|widget| widget.layout(cx));
    }
}

/*
#########################################################
#
# Box<[W]>
#
#########################################################
*/

impl<IV: IntoView> Widget for Box<[IV]> {
    fn build(&self, cx: &mut BuildCx<'_>) {
        self.iter().for_each(|w| w.build(cx));
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.iter().for_each(|w| w.layout(cx));
    }
}

/*
#########################################################
#
# [W; N]
#
#########################################################
*/

impl<IV: IntoView, const N: usize> Widget for [IV; N] {
    fn build(&self, cx: &mut BuildCx<'_>) {
        self.iter().for_each(|w| w.build(cx));
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.iter().for_each(|w| w.layout(cx));
    }
}

/*
#########################################################
#
# Option<IV>
#
#########################################################
*/

// -- Option<IV>
impl<IV: IntoView> Widget for Option<IV> {
    fn build(&self, cx: &mut BuildCx<'_>) {
        match self {
            Some(widget) => widget.build(cx),
            None => {},
        }
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        match self {
            Some(widget) => widget.layout(cx),
            None => {},
        }
    }
}

/*
#########################################################
#
# Void
#
#########################################################
*/

// -- ()
impl Widget for () {
    fn build(&self, _cx: &mut BuildCx<'_>) {}
    fn layout(&self, _cx: &mut LayoutCx<'_>) {}
}

/*
#########################################################
#
# SignalRead<W>
#
#########################################################
*/

impl<IV: IntoView> Widget for SignalRead<IV> {
    fn build(&self, cx: &mut BuildCx<'_>) {
        self.with(|w| w.build(cx))
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.with(|w| w.layout(cx))
    }
}

/*
#########################################################
#
# Signal<W>
#
#########################################################
*/

impl<IV: IntoView> Widget for Signal<IV> {
    fn build(&self, cx: &mut BuildCx<'_>) {
        self.with(|w| w.build(cx))
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.with(|w| w.layout(cx))
    }
}

/*
#########################################################
#
# Memo<W>
#
#########################################################
*/

impl<IV: IntoView + PartialEq> Widget for Memo<IV> {
    fn build(&self, cx: &mut BuildCx<'_>) {
        self.with(|w| w.build(cx))
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.with(|w| w.layout(cx))
    }
}

/*
#########################################################
#
# Text
#
#########################################################
*/

pub fn label(text: impl AsRef<str>) -> Label {
    Label {
        text: Arc::from(text.as_ref())
    }
}

pub struct Label {
    text: Arc<str>
}

pub struct TextState {}

impl Widget for Label {
    fn build(&self, cx: &mut BuildCx<'_>) {
        cx.set_state(TextState {});
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let bound = cx.bound;
        let layout_node = bound;

        match cx.rules.axis {
            Axis::Horizontal => {
                cx.bound.x += layout_node.width + cx.rules.spacing.0 as f32;
            },
            Axis::Vertical =>  {
                cx.bound.y += layout_node.height + cx.rules.spacing.0 as f32;
            },
        }

        cx.set_node(layout_node);
    }
}

impl Widget for &'static str {
    fn build(&self, cx: &mut BuildCx<'_>) {
        cx.set_state(TextState {});
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let bound = cx.bound;
        let layout_node = bound;

        match cx.rules.axis {
            Axis::Horizontal => {
                cx.bound.x += layout_node.width + cx.rules.spacing.0 as f32;
            },
            Axis::Vertical =>  {
                cx.bound.y += layout_node.height + cx.rules.spacing.0 as f32;
            },
        }

        cx.set_node(layout_node);
    }
}

impl Widget for String {
    fn build(&self, cx: &mut BuildCx<'_>) {
        cx.set_state(TextState {});
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let bound = cx.bound;
        let layout_node = bound;

        match cx.rules.axis {
            Axis::Horizontal => {
                cx.bound.x += layout_node.width + cx.rules.spacing.0 as f32;
            },
            Axis::Vertical =>  {
                cx.bound.y += layout_node.height + cx.rules.spacing.0 as f32;
            },
        }

        cx.set_node(layout_node);
    }
}

/*
#########################################################
#
# Integers
#
#########################################################
*/

macro_rules! impl_view_primitive {
    ($name:ty) => {
        impl Widget for $name {
            fn build(&self, cx: &mut BuildCx<'_>) {
                cx.set_state(())
            }

            fn layout(&self, _cx: &mut LayoutCx<'_>) {}
        }
    };

    ($next:ty, $($rest:ty),*) => {
        impl_view_primitive!{ $next }
        impl_view_primitive!{ $($rest),* }
    };
}

impl_view_primitive!(
    u8,    i8,
    u16,   i16,
    u32,   i32,
    u64,   i64,
    usize, isize,
    u128,  i128
);
