use std::sync::Arc;

use aplite_reactive::*;
use aplite_renderer::Scene;
use aplite_types::{
    Color, Length, Matrix3x2, PaintRef, Rect, rgb, rgba
};

use crate::{layout::Axis, state::BorderWidth};
use crate::view::IntoView;
use crate::context::{BuildCx, CursorCx, LayoutCx};

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
pub trait Widget: 'static {
    fn debug_name(&self) -> &'static str {
        let name = std::any::type_name::<Self>();
        name.split("::")
            .into_iter()
            .find(|s| s.contains('<'))
            .and_then(|s| s.split('<').next())
            .unwrap_or(name)
    }

    fn build(&self, cx: &mut BuildCx<'_>);

    fn layout(&self, cx: &mut LayoutCx<'_>);

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool;
}

pub trait Renderable: std::fmt::Debug {
    fn render(&self, rect: &Rect, scene: &mut Scene);
}

impl Renderable for () {
    fn render(&self, _rect: &Rect, _scene: &mut Scene) {}
}

/*
#########################################################
#
# ViewFn
#
#########################################################
*/

pub struct MemoizedView<T, State: PartialEq = bool> {
    pub(crate) view: T,
    pub(crate) state: State,
}

impl<T: Widget, State: PartialEq> MemoizedView<T, State> {
    pub fn new<IV: IntoView<View = T>>(view: IV, state: State) -> Self {
        Self {
            view: view.into_view(),
            state,
        }
    }
}

impl<T, State: PartialEq> PartialEq for MemoizedView<T, State> {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl<T> Widget for MemoizedView<T>
where
    T: Widget,
{
    fn debug_name(&self) -> &'static str {
        self.view.debug_name()
    }

    fn build(&self, cx: &mut BuildCx<'_>) {
        self.view.build(cx);
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.view.layout(cx);
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        self.view.detect_hover(cx)
    }
}

impl<F, IV> IntoView for F
where
    F: Fn() -> IV + 'static,
    IV: IntoView,
{
    // type View = Memo<MemoizedView<IV::View>>;
    type View = Memo<IV::View>;

    fn into_view(self) -> Self::View {
        Memo::with_compare(move |_| self().into_view(), |_, _| false)
        // Memo::new(move |_| MemoizedView::new(self(), false))
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
    VT: Widget,
    VF: Widget,
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

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        match self {
            Either::True(t) => t.detect_hover(cx),
            Either::False(f) => f.detect_hover(cx),
        }
    }
}

impl<VT, VF> IntoView for Either<VT, VF>
where
    VT: IntoView,
    VF: IntoView,
{
    type View = Either<VT::View, VF::View>;

    fn into_view(self) -> Self::View {
        match self {
            Either::True(t) => Either::True(t.into_view()),
            Either::False(f) => Either::False(f.into_view()),
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
        true => Either::True(content_true()),
        false => Either::False(content_false()),
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
    style_fn: Option<Box<dyn Fn(&mut CircleElement)>>,
}

impl CircleWidget {
    pub fn style(self, style_fn: impl Fn(&mut CircleElement) + 'static) -> Self {
        Self {
            style_fn: Some(Box::new(style_fn)),
            ..self
        }
    }
}

impl IntoView for CircleWidget {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

impl Widget for CircleWidget {
    fn build(&self, cx: &mut BuildCx<'_>) {
        let mut state = CircleElement::new();
        if let Some(style_fn) = self.style_fn.as_ref() {
            style_fn(&mut state);
        }

        cx.register_element(state);
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let state = cx.get_element::<CircleElement>().unwrap();
        let bound = cx.bound;

        let radius = match state.radius {
            Length::Grow => bound.width.max(bound.height),
            Length::Fixed(val) => val,
            Length::FitContent => 0.,
        };

        let node = Rect::new(bound.x, bound.y, radius, radius);

        match cx.rules.axis {
            Axis::Horizontal => {
                cx.bound.x += radius + cx.rules.spacing.0 as f32;
            },
            Axis::Vertical =>  {
                cx.bound.y += radius + cx.rules.spacing.0 as f32;
            },
        }

        cx.set_node(node);
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        let rect = cx.get_layout_node().unwrap();
        rect.contains(cx.hover_pos())
    }
}

#[derive(PartialEq, Eq)]
pub struct CircleElement {
    pub radius: Length,
    pub background: Color,
    pub border_color: Color,
    pub border_width: BorderWidth,
}

impl std::fmt::Debug for CircleElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircleElement")
            .finish_non_exhaustive()
    }
}

impl CircleElement {
    fn new() -> Self {
        Self {
            radius: Length::Grow,
            background: rgba(0xff6969ff),
            border_color: rgb(0x000000),
            border_width: BorderWidth(0.),
        }
    }
}

impl Renderable for CircleElement {
    fn render(&self, rect: &Rect, scene: &mut Scene) {
        scene.draw_circle(
            rect,
            &Matrix3x2::identity(),
            &PaintRef::from(&self.background),
            &PaintRef::from(&self.border_color),
            &self.border_width.0
        );
    }
}

/*
#########################################################
#
# Iterables
#
#########################################################
*/

macro_rules! build {
    ($name:ident, $cx:ident) => {
        let mut path_id = $cx.pop();

        $name.iter().for_each(|w| {
            $cx.with_id(path_id, |cx| w.build(cx));
            path_id += 1;
        });

        $cx.push(path_id);
    };
}

macro_rules! layout {
    ($name:ident, $cx:ident) => {
        let count = $name.len();

        let bound = match $cx.rules.axis {
            Axis::Horizontal => {
                let width = $cx.bound.width / count as f32;
                Rect::new($cx.bound.x, $cx.bound.y, width, $cx.bound.height)
            },
            Axis::Vertical => {
                let height = $cx.bound.height / count as f32;
                Rect::new($cx.bound.x, $cx.bound.y, $cx.bound.width, height)
            },
        };

        let mut cx = LayoutCx::new($cx.cx, $cx.rules, bound);

        let mut path_id = cx.pop();

        $name.iter().for_each(|w| {
            cx.with_id(path_id, |cx| w.layout(cx));
            path_id += 1;
        });

        cx.push(path_id);
    };
}

fn detect_hover<'a, T: Widget>(mut name: impl Iterator<Item = &'a T>, cx: &mut CursorCx<'_>) -> bool {
    let mut id_path = cx.pop();

    let any = name.any(|widget| {
        let res = cx.with_id(id_path, |cx| widget.detect_hover(cx));
        id_path += 1;
        res
    });

    cx.push(id_path);
    any
}

impl<T: Widget> Widget for Vec<T> {
    fn build(&self, cx: &mut BuildCx<'_>) {
        build!(self, cx);
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        layout!(self, cx);
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        detect_hover(self.iter(), cx)
    }
}

impl<T: Widget> IntoView for Vec<T> {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

impl<T: Widget> Widget for Box<[T]> {
    fn build(&self, cx: &mut BuildCx<'_>) {
        build!(self, cx);
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        layout!(self, cx);
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        detect_hover(self.iter(), cx)
    }
}

impl<T: Widget + 'static> IntoView for Box<[T]> {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

impl<T: Widget + 'static, const N: usize> Widget for [T; N] {
    fn build(&self, cx: &mut BuildCx<'_>) {
        build!(self, cx);
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        layout!(self, cx);
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        detect_hover(self.iter(), cx)
    }
}

impl<T: Widget + 'static, const N: usize> IntoView for [T; N] {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
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
impl<T: Widget + 'static> Widget for Option<T> {
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

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        match self {
            Some(widget) => {
                widget.detect_hover(cx)
            },
            None => false,
        }
    }
}

impl<IV: IntoView> IntoView for Option<IV> {
    type View = Option<IV::View>;

    fn into_view(self) -> Self::View {
        match self {
            Some(w) => Some(w.into_view()),
            None => None,
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
    fn detect_hover(&self, _cx: &mut CursorCx<'_>) -> bool { false }
}

impl IntoView for () {
    type View = Self;

    fn into_view(self) -> Self::View {}
}

/*
#########################################################
#
# ReactiveNodes
#
#########################################################
*/

macro_rules! impl_reactive_nodes {
    ($name:ident <$generics:tt> where $($where_clause:tt)*) => {
        impl<$generics> Widget for $name<$generics>
        where $($where_clause)?
        {
            fn debug_name(&self) -> &'static str {
                self.with_untracked(|w| w.debug_name())
            }

            fn build(&self, cx: &mut BuildCx<'_>) {
                self.with_untracked(|w| w.build(cx))
            }

            fn layout(&self, cx: &mut LayoutCx<'_>) {
                self.with_untracked(|w| w.layout(cx))
            }

            fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
                self.with_untracked(|w| w.detect_hover(cx))
            }
        }

        impl<$generics> IntoView for $name<$generics>
        where $($where_clause)? {
            type View = Self;

            fn into_view(self) -> Self::View {
                self
            }
        }
    };
}

impl_reactive_nodes!(SignalRead<IV> where IV: Widget);
impl_reactive_nodes!(Signal<IV> where IV: Widget);
impl_reactive_nodes!(Memo<IV> where IV: Widget);

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

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text.as_ref())
    }
}

macro_rules! impl_text {
    ($name:ty) => {
        impl Widget for $name {
            fn build(&self, cx: &mut BuildCx<'_>) {
                let text = self.to_string();

                cx.register_element(TextElement {
                    len: text.len(),
                    color: rgb(0x000000),
                });
            }

            fn layout(&self, cx: &mut LayoutCx<'_>) {
                let element = cx.get_element::<TextElement>().unwrap();
                let len = element.len as f32;

                let node = match cx.rules.axis {
                    Axis::Horizontal => {
                        let width = cx.bound.width.min(len);
                        cx.bound.x += width + cx.rules.spacing.0 as f32;
                        Rect::new(cx.bound.x, cx.bound.y, width, cx.bound.height)
                    },
                    Axis::Vertical =>  {
                        let height = cx.bound.height.min(len);
                        cx.bound.y += height + cx.rules.spacing.0 as f32;
                        Rect::new(cx.bound.x, cx.bound.y, cx.bound.width, height)
                    },
                };

                cx.set_node(node);
            }

            fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
                let rect = cx.get_layout_node().unwrap();
                rect.contains(cx.hover_pos())
            }
        }

        impl IntoView for $name {
            type View = Self;

            fn into_view(self) -> Self::View {
                self
            }
        }
    };

    ($next:ty, $($rest:ty),*) => {
        impl_text!{ $next }
        impl_text!{ $($rest),* }
    };
}

impl_text!(
    u8,    i8,
    u16,   i16,
    u32,   i32,
    u64,   i64,
    usize, isize,
    u128,  i128,
    &'static str,
    String,
    Label
);

#[derive(PartialEq, Eq)]
pub struct TextElement {
    pub len: usize,
    pub color: Color,
}

impl std::fmt::Debug for TextElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextElement")
            .finish_non_exhaustive()
    }
}

impl Renderable for TextElement {
    fn render(&self, _rect: &Rect, _scene: &mut Scene) {}
}
