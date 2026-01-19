use std::sync::Arc;

use aplite_reactive::*;
use aplite_renderer::Scene;
use aplite_types::{
    Color, Length, Matrix3x2, PaintRef, Rect, rgb, rgba
};

use crate::{layout::Axis, state::BorderWidth};
use crate::layout::LayoutCx;
use crate::view::IntoView;
use crate::context::{BuildCx, Context};

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

    fn detect_hover(&self, cx: &mut Context);
}

pub trait Renderable {
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

    fn detect_hover(&self, cx: &mut Context) {
        self().detect_hover(cx);
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
    pub background: Color,
    pub border_color: Color,
    pub border_width: BorderWidth,
}

impl CircleState {
    fn new() -> Self {
        Self {
            radius: Length::Grow,
            background: rgba(0xff6969ff),
            border_color: rgb(0x000000),
            border_width: BorderWidth(0.),
        }
    }
}

impl Renderable for CircleState {
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

impl Widget for CircleWidget {
    fn build(&self, cx: &mut BuildCx<'_>) {
        let mut state = CircleState::new();
        if let Some(style_fn) = self.style_fn.as_ref() {
            style_fn(&mut state);
        }

        cx.set_state(state);
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let state = cx.get_state::<CircleState>().unwrap();
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

    fn detect_hover(&self, cx: &mut Context) {
        let rect = cx.get_layout_node().unwrap();
        if rect.contains(&cx.cursor.hover.pos) {}
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

    fn detect_hover(&self, cx: &mut Context) {
        let rect = cx.get_layout_node().unwrap();
        if rect.contains(&cx.cursor.hover.pos) {}
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
# Iterables
#
#########################################################
*/

macro_rules! handle_iterables {
    ($name:ident, $cx:ident, $cmd:ident) => {
        let mut path_id = $cx.pop();
        $name.iter().for_each(|w| {
            $cx.with_id(path_id, |cx| w.$cmd(cx));
            path_id += 1;
        });
        $cx.push(path_id);
    };
}

impl<IV: IntoView> Widget for Vec<IV> {
    fn build(&self, cx: &mut BuildCx<'_>) {
        handle_iterables!(self, cx, build);
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        handle_iterables!(self, cx, layout);
    }

    fn detect_hover(&self, cx: &mut Context) {
        let rect = cx.get_layout_node().unwrap();
        if rect.contains(&cx.cursor.hover.pos) {}
    }
}

impl<IV: IntoView> Widget for Box<[IV]> {
    fn build(&self, cx: &mut BuildCx<'_>) {
        handle_iterables!(self, cx, build);
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        handle_iterables!(self, cx, layout);
    }

    fn detect_hover(&self, cx: &mut Context) {
        let rect = cx.get_layout_node().unwrap();
        if rect.contains(&cx.cursor.hover.pos) {}
    }
}

impl<IV: IntoView, const N: usize> Widget for [IV; N] {
    fn build(&self, cx: &mut BuildCx<'_>) {
        handle_iterables!(self, cx, build);
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        handle_iterables!(self, cx, layout);
    }

    fn detect_hover(&self, cx: &mut Context) {
        let rect = cx.get_layout_node().unwrap();
        if rect.contains(&cx.cursor.hover.pos) {}
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

    fn detect_hover(&self, cx: &mut Context) {
        let rect = cx.get_layout_node().unwrap();
        if rect.contains(&cx.cursor.hover.pos) {}
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
    fn detect_hover(&self, _cx: &mut Context) {}
}

/*
#########################################################
#
# ReactiveNodes
#
#########################################################
*/

macro_rules! impl_reactive_nodes {
    ($name:ident <$($generics:tt)?> where $($where_clause:tt)*) => {
        impl<$($generics,)?> Widget for $name<$($generics,)?>
        where $($where_clause)?
        {
            fn build(&self, cx: &mut BuildCx<'_>) {
                self.with_untracked(|w| w.build(cx))
            }

            fn layout(&self, cx: &mut LayoutCx<'_>) {
                self.with_untracked(|w| w.layout(cx))
            }

            fn detect_hover(&self, cx: &mut Context) {
                self.with_untracked(|w| w.detect_hover(cx))
            }
        }
    };
}

impl_reactive_nodes!(SignalRead<IV> where IV: IntoView);
impl_reactive_nodes!(Signal<IV> where IV: IntoView);
impl_reactive_nodes!(Memo<IV> where IV: IntoView + PartialEq);

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

impl Renderable for TextState {
    fn render(&self, _rect: &Rect, _scene: &mut Scene) {}
}

macro_rules! impl_text {
    ($(&$lifetime:lifetime)? $name:ident) => {
        impl Widget for $(&$lifetime)? $name {
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

            fn detect_hover(&self, cx: &mut Context) {
                let rect = cx.get_layout_node().unwrap();
                if rect.contains(&cx.cursor.hover.pos) {}
            }
        }
    };
}

impl_text!(Label);
impl_text!(String);
impl_text!(&'static str);

/*
#########################################################
#
# Integers
#
#########################################################
*/

macro_rules! impl_integers {
    ($name:ty) => {
        impl Widget for $name {
            fn build(&self, cx: &mut BuildCx<'_>) {
                cx.set_state(TextState {})
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

            fn detect_hover(&self, cx: &mut Context) {
                let rect = cx.get_layout_node().unwrap();
                if rect.contains(&cx.cursor.hover.pos) {}
            }
        }
    };

    ($next:ty, $($rest:ty),*) => {
        impl_integers!{ $next }
        impl_integers!{ $($rest),* }
    };
}

impl_integers!(
    u8,    i8,
    u16,   i16,
    u32,   i32,
    u64,   i64,
    usize, isize,
    u128,  i128
);
