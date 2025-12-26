use aplite_reactive::*;
use aplite_renderer::Scene;
use aplite_storage::{Component, make_component};
use aplite_types::{Matrix3x2, Rgba};

// use crate::layout::*;
use crate::view::{AnyView, IntoView, View};
use crate::context::Context;
// use crate::callback::WidgetEvent;

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
    #[cfg(debug_assertions)]
    fn debug_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn layout(&self, cx: &mut Context);

    fn draw(&self, scene: &mut Scene);
}

pub trait Mountable: Sized {
    fn build(self, cx: &mut Context);
}

pub trait InteractiveWidget: Widget {
    fn execute(&self) {}
}

/*
#########################################################
#
# Circle
#
#########################################################
*/

pub fn circle() -> impl IntoView {
    CircleWidget {
        radius: Radius(100.),
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Radius(f32);

make_component!(Radius);

#[derive(Clone, PartialEq)]
pub struct CircleWidget {
    radius: Radius,
}

impl Widget for CircleWidget {
    fn layout(&self, cx: &mut Context) {
       todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

impl Mountable for CircleWidget {
    fn build(self, cx: &mut Context) {
    }
}

/*
#########################################################
#
# Show
#
#########################################################
*/

enum Show<IV1: IntoView, IV2: IntoView> {
    True(IV1),
    False(IV2)
}

impl<IV1: IntoView, IV2: IntoView> Widget for Show<IV1, IV2> {
    fn layout(&self, cx: &mut Context) {
        match self {
            Show::True(iv1) => iv1.layout(cx),
            Show::False(iv2) => iv2.layout(cx),
        }
    }

    fn draw(&self, scene: &mut Scene) {
        match self {
            Show::True(iv1) => iv1.draw(scene),
            Show::False(iv2) => iv2.draw(scene),
        }
    }
}

impl<IV1: IntoView, IV2: IntoView> Mountable for Show<IV1, IV2> {
    fn build(self, cx: &mut Context) {
        match self {
            Show::True(iv1) => iv1.build(cx),
            Show::False(iv2) => iv2.build(cx),
        }
    }
}

pub fn show<TrueFn, FalseFn, W, TrueIV, FalseIV>(
    content_true: TrueFn,
    content_false: FalseFn,
    when: W,
) -> impl IntoView
where
    TrueFn: Fn() -> TrueIV + 'static,
    FalseFn: Fn() -> FalseIV + 'static,
    W: Fn() -> bool + 'static,
    TrueIV: IntoView,
    FalseIV: IntoView,
{
    let when = Memo::new(move |_| when());

    move || match when.get() {
        true => Show::True(content_true),
        false => Show::False(content_false),
    }
}

/*
#########################################################
#
# Other Types
#
#########################################################
*/

// -- ViewFn
impl<F, IV> Widget for F
where
    F: FnOnce() -> IV,
    IV: IntoView,
{
    fn layout(&self, _: &mut Context) {}

    fn draw(&self, _: &mut Scene) {}
}

impl<F, IV> Mountable for F
where
    F: FnOnce() -> IV + 'static,
    IV: IntoView,
{
    fn build(self, cx: &mut Context) {
        self().into_view().build(cx);
    }
}

// -- Vec<IV>
impl<IV: IntoView> Widget for Vec<IV> {
    fn layout(&self, cx: &mut Context) {
        self.iter().for_each(|widget| widget.layout(cx));
    }

    fn draw(&self, scene: &mut Scene) {
        self.iter().for_each(|widget| widget.draw(scene));
    }
}

// -- Option<IV>
impl<IV: IntoView> Widget for Option<IV> {
    fn layout(&self, cx: &mut Context) {
        match self {
            Some(widget) => widget.layout(cx),
            None => {},
        }
    }

    fn draw(&self, scene: &mut Scene) {
        match self {
            Some(widget) => widget.draw(scene),
            None => {},
        }
    }
}

// -- ()
impl Widget for () {
    fn layout(&self, _: &mut Context) {}
    fn draw(&self, _: &mut Scene) {}
}

impl Mountable for () {
    fn build(self, _: &mut Context) {}
}

impl<IV: IntoView> Mountable for Option<IV> {
    fn build(self, cx: &mut Context) {
        match self {
            Some(widget) => widget.build(cx),
            None => {},
        }
    }
}

// -- Signal
impl<IV: IntoView> Widget for SignalRead<IV> {
    fn layout(&self, cx: &mut Context) {
        self.with(|widget| widget.layout(cx))
    }
    fn draw(&self, scene: &mut Scene) {
        self.with(|widget| widget.draw(scene))
    }
}

impl<IV: IntoView + Clone> Mountable for SignalRead<IV> {
    fn build(self, cx: &mut Context) {
        self.get().build(cx);
    }
}

// -- Primitives
macro_rules! impl_view_primitive {
    ($name:ty) => {
        impl Widget for $name {
            fn layout(&self, _: &mut Context) {}

            fn draw(&self, _: &mut Scene) {}
        }

        impl Mountable for $name {
            fn build(self, _: &mut Context) {}
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
