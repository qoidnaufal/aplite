use aplite_reactive::*;
use aplite_renderer::Scene;
use aplite_types::{Length, Matrix3x2, PaintRef, Rect, Rgba, Size, Vec2f, rgb, rgba};

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

pub struct WidgetElement {}

/*
#########################################################
#
# Widget Trait
#
#########################################################
*/

/// main building block to create a renderable component
pub trait Widget: 'static {
    fn debug_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn width(&self) -> Length {
        Length::Grow
    }

    fn height(&self) -> Length {
        Length::Grow
    }

    fn layout_node_size(&self, bound: Size) -> Size {
        Size::default()
    }

    fn layout(&self, cx: &mut LayoutCx<'_>);

    fn draw(&self, scene: &mut Scene);
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

    fn layout_node_size(&self, bound: Size) -> Size {
        self().into_view().layout_node_size(bound)
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self().into_view().layout(cx);
    }

    fn draw(&self, scene: &mut Scene) {
        self().into_view().draw(scene);
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
        background: Background(rgba(0xff6969ff)),
        border_color: BorderColor(rgb(0x000000)),
        border_width: BorderWidth(0.),
    }
}

#[derive(Clone, PartialEq)]
pub struct CircleWidget {
    pos: Vec2f,
    radius: Length,
    background: Background,
    border_color: BorderColor,
    border_width: BorderWidth,
}

impl CircleWidget {
    pub fn radius(self, radius: Length) -> Self {
        Self { radius, ..self }
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
    fn width(&self) -> Length {
        self.radius
    }

    fn height(&self) -> Length {
        self.radius
    }

    fn layout_node_size(&self, bound: Size) -> Size {
        Size::default()
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let _ = cx;
    }

    fn draw(&self, scene: &mut Scene) {
        scene.draw_circle(
            &Rect::default(),
            &Matrix3x2::identity(),
            &PaintRef::Color(&self.background.0),
            &PaintRef::Color(&self.background.0),
            &0.
        );
    }
}

impl IntoView for CircleWidget {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
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

    fn layout_node_size(&self, bound: Size) -> Size {
        match self {
            Either::True(iv1) => iv1.layout_node_size(bound),
            Either::False(iv2) => iv2.layout_node_size(bound),
        }
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
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

impl<VT, VF> IntoView for Either<VT, VF>
where
    VT: IntoView,
    VF: IntoView,
{
    type View = Self;

    fn into_view(self) -> Self::View {
        self
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
        true => Either::True(content_true()),
        false => Either::False(content_false()),
    }
}

/*
#########################################################
#
# Vec<W>
#
#########################################################
*/

impl<W: Widget> Widget for Vec<W> {
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.iter().for_each(|widget| widget.layout(cx));
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

impl<W: Widget + 'static> Widget for Box<[W]> {
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.for_each(|w| w.layout(cx));
    }

    fn draw(&self, scene: &mut Scene) {
        self.for_each(|w| w.draw(scene));
    }
}

impl<W: Widget + 'static> IntoView for Box<[W]> {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

impl<W: Widget + 'static> ForEachView for Box<[W]> {
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

impl<W: Widget + 'static, const N: usize> Widget for [W; N] {
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.for_each(|w| w.layout(cx));
    }

    fn draw(&self, scene: &mut Scene) {
        self.for_each(|w| w.draw(scene));
    }
}

impl<W: Widget + 'static, const N: usize> IntoView for [W; N] {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

impl<W: Widget + 'static, const N: usize> ForEachView for [W; N] {
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
impl<W: Widget> Widget for Option<W> {
    fn layout(&self, cx: &mut LayoutCx<'_>) {
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
    fn layout(&self, _: &mut LayoutCx<'_>) {}
    fn draw(&self, _: &mut Scene) {}
}

impl IntoView for () {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

/*
#########################################################
#
# SignalRead<W>
#
#########################################################
*/

impl<W: Widget + 'static> Widget for SignalRead<W> {
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.with(|w| w.layout(cx))
    }

    fn draw(&self, scene: &mut Scene) {
        self.with(|w| w.draw(scene))
    }
}

impl<W: Widget + 'static> IntoView for SignalRead<W> {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

/*
#########################################################
#
# Signal<W>
#
#########################################################
*/

impl<W: Widget + 'static> Widget for Signal<W> {
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.with(|w| w.layout(cx))
    }

    fn draw(&self, scene: &mut Scene) {
        self.with(|w| w.draw(scene))
    }
}

impl<W: Widget + 'static> IntoView for Signal<W> {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

/*
#########################################################
#
# Memo<W>
#
#########################################################
*/

impl<W: Widget + PartialEq + 'static> Widget for Memo<W> {
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.with(|w| w.layout(cx))
    }

    fn draw(&self, scene: &mut Scene) {
        self.with(|w| w.draw(scene))
    }
}

impl<W: Widget + PartialEq + 'static> IntoView for Memo<W> {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
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

    fn draw(&self, _: &mut Scene) {}
}

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
            fn layout(&self, _: &mut LayoutCx<'_>) {}

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
