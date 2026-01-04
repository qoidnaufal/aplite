use aplite_reactive::*;
use aplite_renderer::Scene;
use aplite_types::{Length, Matrix3x2, PaintRef, Rect, Rgba, Size, Vec2f, rgb, rgba};
use aplite_storage::Entity;

use crate::state::{Background, BorderColor, BorderWidth, Radius};
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
        let max = bound.width.max(bound.height);
        let min = bound.width.min(bound.height);
        let radius = match self.radius {
            Length::Grow => min,
            Length::Fixed(val) => val.clamp(min, max),
            Length::MinContent(val) => val,
        };
        Size::square(radius)
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

impl<IV: IntoView> Widget for Vec<IV> {
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.iter().for_each(|widget| widget.layout(cx));
    }

    fn draw(&self, scene: &mut Scene) {
        self.iter().for_each(|widget| widget.draw(scene));
    }
}

impl<IV: IntoView> IntoView for Vec<IV> {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
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

    fn draw(&self, scene: &mut Scene) {
        self.for_each(|w| w.draw(scene));
    }
}

impl<IV: IntoView> IntoView for Box<[IV]> {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
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

    fn draw(&self, scene: &mut Scene) {
        self.for_each(|w| w.draw(scene));
    }
}

impl<IV: IntoView, const N: usize> IntoView for [IV; N] {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
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

    fn draw(&self, scene: &mut Scene) {
        match self {
            Some(widget) => widget.draw(scene),
            None => {},
        }
    }
}

impl<IV: IntoView> IntoView for Option<IV> {
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

impl<IV: IntoView> Widget for SignalRead<IV> {
    fn layout_node_size(&self, bound: Size) -> Size {
        self.with(|w| w.layout_node_size(bound))
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.with(|w| w.layout(cx))
    }

    fn draw(&self, scene: &mut Scene) {
        self.with(|w| w.draw(scene))
    }
}

impl<IV: IntoView> IntoView for SignalRead<IV> {
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

impl<IV: IntoView> Widget for Signal<IV> {
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.with(|w| w.layout(cx))
    }

    fn draw(&self, scene: &mut Scene) {
        self.with(|w| w.draw(scene))
    }
}

impl<IV: IntoView> IntoView for Signal<IV> {
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

impl<IV: IntoView + PartialEq> Widget for Memo<IV> {
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.with(|w| w.layout(cx))
    }

    fn draw(&self, scene: &mut Scene) {
        self.with(|w| w.draw(scene))
    }
}

impl<IV: IntoView + PartialEq> IntoView for Memo<IV> {
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
