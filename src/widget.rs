use aplite_reactive::*;
use aplite_types::{Length, Color, Vec2f, rgb, rgba};

use crate::state::BorderWidth;
use crate::layout::LayoutCx;
use crate::view::{ForEachView, IntoView};
use crate::context::Context;

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

    fn layout(&self, cx: &mut LayoutCx<'_>);
}

pub trait InteractiveWidget: Widget {
    fn trigger(&self) {}
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
        self().into_view().debug_name()
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self().into_view().layout(cx);
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
        pos: Vec2f::default(),
        radius: Length::Grow,
        background: rgba(0xff6969ff),
        border_color: rgb(0x000000),
        border_width: BorderWidth(0.),
    }
}

#[derive(Clone, PartialEq)]
pub struct CircleWidget {
    pos: Vec2f,
    radius: Length,
    background: Color,
    border_color: Color,
    border_width: BorderWidth,
}

impl CircleWidget {
    pub fn radius(self, radius: Length) -> Self {
        Self { radius, ..self }
    }

    pub fn background(self, background: Color) -> Self {
        Self {
            background,
            ..self
        }
    }

    pub fn border_color(self, border_color: Color) -> Self {
        Self {
            border_color,
            ..self
        }
    }

    pub fn border_width(self, border_width: f32) -> Self {
        Self {
            border_width: BorderWidth(border_width),
            ..self
        }
    }
}

impl Widget for CircleWidget {
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let _ = cx;
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
            Either::True(iv1) => iv1.debug_name(),
            Either::False(iv2) => iv2.debug_name(),
        }
    }
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        match self {
            Either::True(iv1) => iv1.layout(cx),
            Either::False(iv2) => iv2.layout(cx),
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
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.iter().for_each(|widget| widget.layout(cx));
    }
}

impl<IV: IntoView> ForEachView for Vec<IV> {
    fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
        self.iter().for_each(|w| f(w));
    }

    fn for_each_mut(&mut self, mut f: impl FnMut(&mut dyn Widget)) {
        self.iter_mut().for_each(|w| f(w));
    }

    fn count(&self) -> usize {
        self.len()
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
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.for_each(|w| w.layout(cx));
    }
}

impl<IV: IntoView> ForEachView for Box<[IV]> {
    fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
        self.iter().for_each(|w| f(w));
    }

    fn for_each_mut(&mut self, mut f: impl FnMut(&mut dyn Widget)) {
        self.iter_mut().for_each(|w| f(w));
    }

    fn count(&self) -> usize {
        self.len()
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
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.for_each(|w| w.layout(cx));
    }
}

impl<IV: IntoView, const N: usize> ForEachView for [IV; N] {
    fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
        self.iter().for_each(|w| f(w));
    }

    fn for_each_mut(&mut self, mut f: impl FnMut(&mut dyn Widget)) {
        self.iter_mut().for_each(|w| f(w));
    }

    fn count(&self) -> usize {
        N
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
    fn layout(&self, _: &mut LayoutCx<'_>) {}
}

/*
#########################################################
#
# SignalRead<W>
#
#########################################################
*/

impl<IV: IntoView> Widget for SignalRead<IV> {
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

impl Widget for &'static str {
    fn layout(&self, _: &mut LayoutCx<'_>) {}
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
            fn layout(&self, _: &mut LayoutCx<'_>) {}
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
