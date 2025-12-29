use aplite_reactive::*;
use aplite_renderer::Scene;
use aplite_types::{Matrix3x2, PaintRef, Rect, Rgba, Size, Vec2f, rgba, rgb};

use crate::layout::Axis;
use crate::state::{Background, BorderColor, BorderWidth, Radius};
use crate::layout::LayoutCx;
// use crate::layout::*;
use crate::view::{ForEachView, IntoView};
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

    fn layout_node_size(&self, orientation: Axis) -> Size {
        let _ = orientation;
        Size::default()
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>);

    fn draw(&self, scene: &mut Scene);
}

pub trait WidgetExt: IntoView {
    fn style(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl<IV: IntoView> WidgetExt for IV {}

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

pub fn circle() -> CircleWidget {
    CircleWidget {
        pos: Vec2f::default(),
        radius: Radius(100.),
        background: Background(rgba(0xff6969ff)),
        border_color: BorderColor(rgb(0x000000)),
        border_width: BorderWidth(0.),
    }
}

#[derive(Clone, PartialEq)]
pub struct CircleWidget {
    pos: Vec2f,
    radius: Radius,
    background: Background,
    border_color: BorderColor,
    border_width: BorderWidth,
}

impl CircleWidget {
    pub fn radius(self, radius: f32) -> Self {
        Self {
            radius: Radius(radius),
            ..self
        }
    }

    pub fn background(self, background: Rgba) -> Self {
        Self {
            background: Background(background),
            ..self
        }
    }

    pub fn border_color(self, border_color: Rgba) -> Self {
        Self {
            border_color: BorderColor(border_color),
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
    fn layout_node_size(&self, _: Axis) -> Size {
        Size::new(self.radius.0, self.radius.0)
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) {
        self.pos = cx.get_next_pos(Size::new(self.radius.0, self.radius.0));
    }

    fn draw(&self, scene: &mut Scene) {
        scene.draw_circle(
            &Rect::from_vec2f_size(self.pos, Size::new(self.radius.0, self.radius.0)),
            &Matrix3x2::identity(),
            &PaintRef::Color(&self.background.0),
            &PaintRef::Color(&self.background.0),
            &0.
        );
    }
}

impl ForEachView for CircleWidget {}

impl IntoView for CircleWidget {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

/*
#########################################################
#
# Either & ViewFn
#
#########################################################
*/

pub enum Either<T, F> {
    True(T),
    False(F)
}

impl<T, F> Widget for Either<T, F>
where
    T: IntoView,
    F: IntoView,
{
    fn debug_name(&self) -> &'static str {
        match self {
            Either::True(iv1) => iv1.debug_name(),
            Either::False(iv2) => iv2.debug_name(),
        }
    }

    fn layout_node_size(&self, orientation: Axis) -> Size {
        match self {
            Either::True(iv1) => iv1.layout_node_size(orientation),
            Either::False(iv2) => iv2.layout_node_size(orientation),
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) {
        match self {
            Either::True(iv1) => iv1.layout(cx),
            Either::False(iv2) => iv2.layout(cx),
        }
    }

    fn draw(&self, scene: &mut Scene) {
        match self {
            Either::True(iv1) => iv1.draw(scene),
            Either::False(iv2) => iv2.draw(scene),
        }
    }
}

impl<T, F> ForEachView for Either<T, F>
where
    T: IntoView,
    F: IntoView, {}

impl<T, F> IntoView for Either<T, F>
where
    T: IntoView,
    F: IntoView,
{
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

pub fn either<W, TrueFn, FalseFn, T, F>(
    when: W,
    content_true: TrueFn,
    content_false: FalseFn,
) -> impl IntoView
where
    W: Fn() -> bool + 'static,
    TrueFn: Fn() -> T + 'static,
    FalseFn: Fn() -> F + 'static,
    T: IntoView,
    F: IntoView,
{
    let when = Memo::new(move |_| when());

    move || match when.get() {
        true => Either::True(content_true().into_view()),
        false => Either::False(content_false().into_view()),
    }
}

impl<F, IV> Widget for F
where
    F: Fn() -> IV + 'static,
    IV: IntoView,
{
    fn debug_name(&self) -> &'static str {
        let w = self();
        w.debug_name()
    }

    fn layout_node_size(&self, orientation: Axis) -> Size {
        let w = self();
        w.layout_node_size(orientation)
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) {
        let mut w = self();
        w.layout(cx);
    }

    fn draw(&self, scene: &mut Scene) {
        self().draw(scene);
    }
}

impl<F, IV> IntoView for F
where
    F: Fn() -> IV + 'static,
    IV: IntoView,
{
    type View = IV::View;

    fn into_view(self) -> Self::View {
        self().into_view()
    }
}

impl<F, IV> ForEachView for F
where
    F: Fn() -> IV + 'static,
    IV: IntoView,
{}

/*
#########################################################
#
# Vec<IV>
#
#########################################################
*/

// -- Vec<IV>
impl<W: Widget> Widget for Vec<W> {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) {
        self.iter_mut().for_each(|widget| widget.layout(cx));
    }

    fn draw(&self, scene: &mut Scene) {
        self.iter().for_each(|widget| widget.draw(scene));
    }
}

impl<W: Widget + 'static> IntoView for Vec<W> {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

impl<W: Widget + 'static> ForEachView for Vec<W> {
    fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
        self.iter().for_each(|w| f(w));
    }

    fn for_each_mut(&mut self, mut f: impl FnMut(&mut dyn Widget)) {
        self.iter_mut().for_each(|w| f(w));
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
impl<W: Widget> Widget for Option<W> {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) {
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

impl<W: Widget> ForEachView for Option<W> {
    fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
        if let Some(w) = self {
            f(w)
        }
    }

    fn for_each_mut(&mut self, mut f: impl FnMut(&mut dyn Widget)) {
        if let Some(w) = self {
            f(w)
        }
    }
}

impl<W: Widget + 'static> IntoView for Option<W> {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
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
    fn layout(&mut self, _: &mut LayoutCx<'_>) {}
    fn draw(&self, _: &mut Scene) {}
}

impl ForEachView for () {}

impl IntoView for () {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

/*
#########################################################
#
# Reactive Nodes
#
#########################################################
*/

impl<IV: IntoView + Clone> Widget for SignalRead<IV> {
    fn layout(&mut self, _: &mut LayoutCx<'_>) {}
    fn draw(&self, scene: &mut Scene) {
        self.with_untracked(|w| w.draw(scene))
    }
}

impl<IV: IntoView + Clone> ForEachView for SignalRead<IV> {
    fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
        self.with_untracked(|w| f(w))
    }

    fn for_each_mut(&mut self, _: impl FnMut(&mut dyn Widget)) {
    }
}

impl<IV: IntoView + Clone> IntoView for SignalRead<IV> {
    type View = IV::View;

    fn into_view(self) -> Self::View {
        self.get_untracked().into_view()
    }
}

// ----

impl<IV: IntoView + Clone> Widget for Signal<IV> {
    fn layout(&mut self, _: &mut LayoutCx<'_>) {}
    fn draw(&self, scene: &mut Scene) {
        self.with_untracked(|w| w.draw(scene))
    }
}

impl<IV: IntoView + Clone> ForEachView for Signal<IV> {
    fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
        self.with_untracked(|w| f(w))
    }
}

impl<IV: IntoView + Clone> IntoView for Signal<IV> {
    type View = IV::View;

    fn into_view(self) -> Self::View {
        self.get_untracked().into_view()
    }
}

impl<IV: IntoView + Clone + PartialEq> Widget for Memo<IV> {
    fn layout(&mut self, _: &mut LayoutCx<'_>) {
    }

    fn draw(&self, scene: &mut Scene) {
        self.with_untracked(|w| w.draw(scene))
    }
}

impl<IV: IntoView + Clone + PartialEq> ForEachView for Memo<IV> {
    fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
        self.with_untracked(|w| f(w))
    }
}

impl<IV: IntoView + Clone + PartialEq> IntoView for Memo<IV> {
    type View = IV::View;

    fn into_view(self) -> Self::View {
        self.get_untracked().into_view()
    }
}

/*
#########################################################
#
# Text
#
#########################################################
*/

impl<'a> Widget for &'a str {
    fn layout(&mut self, _: &mut LayoutCx<'_>) {}

    fn draw(&self, _: &mut Scene) {}
}

impl<'a> ForEachView for &'a str {}

impl IntoView for &'static str {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
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
            fn layout(&mut self, _: &mut LayoutCx<'_>) {}

            fn draw(&self, _: &mut Scene) {}
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
