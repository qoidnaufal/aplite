use aplite_reactive::*;
use aplite_renderer::Scene;
use aplite_types::{
    Color, Length, Matrix3x2, PaintRef, Rect, theme
};

use crate::{layout::Axis, state::BorderWidth};
use crate::view::IntoView;
use crate::context::{BuildCx, LayoutCx, CursorCx};

mod button;
mod image;
mod stack;
mod either;
mod text;
mod iterables;

pub use {
    button::*,
    image::*,
    stack::*,
    either::*,
    text::*,
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
