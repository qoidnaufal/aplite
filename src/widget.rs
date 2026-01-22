use std::sync::Arc;

use aplite_reactive::*;
use aplite_renderer::Scene;
use aplite_types::{
    Color, Length, Matrix3x2, PaintRef, Rect, rgb, theme
};

use crate::{layout::Axis, state::BorderWidth};
use crate::view::IntoView;
use crate::context::{BuildCx, LayoutCx, CursorCx};

mod button;
mod image;
mod stack;
mod either;

pub use {
    button::*,
    image::*,
    stack::*,
    either::*,
};

/*
#########################################################
#
# Widget Trait
#
#########################################################
*/

pub enum InteractionState {
    Idle,
    Hovered,
    Focused,
    Clicked,
}

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

    fn build(&self, cx: &mut BuildCx<'_>) -> bool;

    fn layout(&self, cx: &mut LayoutCx<'_>);

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool;

    fn diff(&self, prev: &dyn Widget) -> bool {
        use std::any::Any;
        self.type_id() == prev.type_id()
        // false
    }
}

pub trait Renderable: std::fmt::Debug + 'static {
    fn render(&self, rect: &Rect, scene: &mut Scene);

    fn type_id(&self) -> std::any::TypeId;

    fn equal(&self, other: &dyn Renderable) -> bool;
}

impl Renderable for () {
    fn render(&self, _rect: &Rect, _scene: &mut Scene) {}

    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn equal(&self, other: &dyn Renderable) -> bool {
        self.type_id() == other.type_id()
    }
}

/*
#########################################################
#
# Fn
#
#########################################################
*/

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
    fn build(&self, cx: &mut BuildCx<'_>) -> bool {
        let mut state = CircleElement::new();

        if let Some(style_fn) = self.style_fn.as_ref() {
            style_fn(&mut state);
        }

        cx.register_element(state)
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let state = cx.get_element::<CircleElement>().unwrap();
        let bound = cx.bound;

        let radius = match state.radius {
            Length::Grow => bound.width.min(bound.height),
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
        let hovered = rect.contains(cx.hover_pos());
        if hovered {
            cx.set_id();
        }
        hovered
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
            background: theme::gruvbox_dark::RED_0,
            border_color: theme::gruvbox_dark::RED_1,
            border_width: BorderWidth(10.),
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

    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn equal(&self, other: &dyn Renderable) -> bool {
        if other.type_id() == self.type_id() {
            unsafe {
                let ptr = other as *const dyn Renderable as *const Self;
                (&*ptr).eq(self)
            }
        } else {
            false
        }
    }
}

/*
#########################################################
#
# Iterables
#
#########################################################
*/

fn build<'a, T: Widget>(mut name: impl Iterator<Item = &'a T>, cx: &mut BuildCx<'_>) -> bool {
    let mut path_id = cx.pop();

    let dirty = name.any(|widget| {
        let dirty = cx.with_id(path_id, |cx| widget.build(cx));
        path_id += 1;
        dirty
    });

    cx.push(path_id);

    dirty
}

fn layout<'a, T: Widget>(name: &'a [T], cx: &mut LayoutCx<'_>) {
    let count = name.len();

    let bound = match cx.rules.axis {
        Axis::Horizontal => {
            let width = cx.bound.width / count as f32;
            Rect::new(cx.bound.x, cx.bound.y, width, cx.bound.height)
        },
        Axis::Vertical => {
            let height = cx.bound.height / count as f32;
            Rect::new(cx.bound.x, cx.bound.y, cx.bound.width, height)
        },
    };

    let mut cx = LayoutCx::derive(cx, cx.rules, bound);

    let mut path_id = cx.pop();

    name.iter().for_each(|w| {
        cx.with_id(path_id, |cx| w.layout(cx));
        path_id += 1;
    });

    cx.push(path_id);
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
    fn build(&self, cx: &mut BuildCx<'_>) -> bool {
        build(self.iter(), cx)
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        layout(self, cx);
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
    fn build(&self, cx: &mut BuildCx<'_>) -> bool {
        build(self.iter(), cx)
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        layout(self, cx);
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
    fn build(&self, cx: &mut BuildCx<'_>) -> bool {
        build(self.iter(), cx)
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        layout(self, cx);
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
    fn build(&self, cx: &mut BuildCx<'_>) -> bool {
        match self {
            Some(widget) => widget.build(cx),
            None => false,
        }
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        match self {
            Some(widget) => widget.layout(cx),
            None => {},
        }
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        self.as_ref().is_some_and(|w| w.detect_hover(cx))
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
    fn build(&self, _cx: &mut BuildCx<'_>) -> bool { false }
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
                self.with(|w| w.debug_name())
            }

            fn build(&self, cx: &mut BuildCx<'_>) -> bool {
                self.with(|w| w.build(cx))
            }

            fn layout(&self, cx: &mut LayoutCx<'_>) {
                self.with(|w| w.layout(cx))
            }

            fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
                self.with(|w| w.detect_hover(cx))
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
            fn build(&self, cx: &mut BuildCx<'_>) -> bool {
                let text = self.to_string();

                cx.register_element(TextElement {
                    len: text.len(),
                    color: rgb(0x000000),
                })
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
                let hovered = rect.contains(cx.hover_pos());

                if hovered {
                    cx.set_id()
                }

                hovered
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

    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn equal(&self, other: &dyn Renderable) -> bool {
        if other.type_id() == self.type_id() {
            unsafe {
                let ptr = other as *const dyn Renderable as *const Self;
                (&*ptr).eq(self)
            }
        } else {
            false
        }
    }
}
