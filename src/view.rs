use aplite_renderer::Scene;
use aplite_storage::{Component, make_component};

use crate::context::Context;
use crate::widget::{Children, Mountable, Widget};

/*
#########################################################
#
# View
#
#########################################################
*/

#[derive(PartialEq)]
pub struct View<W> {
    widget: W
}

impl<W> View<W> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
        }
    }
}

impl<W: Widget + Mountable + 'static> View<W> {
    pub fn as_ref(&self) -> &dyn Widget {
        &self.widget
    }

    pub fn as_mut(&mut self) -> &mut dyn Widget {
        &mut self.widget
    }

    pub fn as_any_view(self) -> AnyView {
        AnyView::new(Box::new(self.widget))
    }
}

impl<W: Widget + Mountable> Widget for View<W> {
    fn layout(&self, cx: &mut Context) {
        self.widget.layout(cx);
    }

    fn draw(&self, scene: &mut Scene) {
        self.widget.draw(scene);
    }
}

impl<W: Widget + Mountable> Mountable for View<W> {
    fn build(self, cx: &mut Context) {
        self.widget.build(cx);
    }
}

impl<W: Widget + Mountable> std::fmt::Debug for View<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(self.widget.debug_name())
            .finish()
    }
}

/*
#########################################################
#
# AnyView
#
#########################################################
*/

pub struct AnyView {
    pub(crate) widget: Box<dyn Widget>,
}

impl AnyView {
    pub(crate) fn new(widget: Box<dyn Widget>) -> Self {
        Self { widget }
    }

    pub(crate) fn as_ref<'a>(&'a self) -> &'a dyn Widget {
        self.widget.as_ref()
    }

    pub(crate) fn as_mut<'a>(&'a mut self) -> &'a mut dyn Widget {
        self.widget.as_mut()
    }
}

impl Widget for AnyView {
    fn layout(&self, cx: &mut Context) {
        self.as_ref().layout(cx);
    }

    fn draw(&self, scene: &mut aplite_renderer::Scene) {
        self.as_ref().draw(scene);
    }
}

make_component!(AnyView);

/*
#########################################################
#
# IntoView
#
#########################################################
*/

/// Types that automatically implement IntoView are:
/// - any type that implement Widget: `impl Widget for T`,
/// - any type that implement Mount: `impl Mount for T`,
/// - any function that produce IntoView: `FnOnce() -> IV where IV: IntoView` or `fn() -> impl IntoView`
pub trait IntoView: Widget + Mountable + Sized + 'static {
    /// View basically is just a build context for the widget which implements it.
    /// Internally it's a `Box<dyn FnOnce(&mut ViewStorage) -> Entity + 'a>`
    fn into_view(self) -> View<Self>;
}


impl<IV> IntoView for IV where IV: Widget + Mountable + Sized + 'static {
    fn into_view(self) -> View<Self> {
        View::new(self)
    }
}

/*
#########################################################
#
# ViewTuple
#
#########################################################
*/

pub trait ViewTuple: IntoView {
    fn into_typed_children(self) -> Children<Self>;

    fn for_each(&self, f: impl FnMut(&dyn Widget));

    fn for_each_mut(&mut self, f: impl FnMut(&mut dyn Widget));
}

macro_rules! impl_tuple_macro {
    ($macro:ident, $next:tt) => {
        $macro!{$next}
    };
    ($macro:ident, $next:tt, $($rest:tt),*) => {
        $macro!{$next, $($rest),*}
        impl_tuple_macro!{$macro, $($rest),*}
    };
}

macro_rules! view_tuple {
    ($($name:ident),*) => {
        impl<$($name: IntoView),*> ViewTuple for ($($name,)*) {
            // type View = ($($name,)*);

            fn into_typed_children(self) -> Children<Self> {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                Children(($($name,)*))
            }

            fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                ($(f($name),)*);
            }

            fn for_each_mut(&mut self, mut f: impl FnMut(&mut dyn Widget)) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                ($(f($name),)*);
            }
        }

        impl<$($name: IntoView),*> Widget for ($($name,)*) {
            fn layout(&self, cx: &mut Context) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                ($($name.layout(cx),)*);
            }

            fn draw(&self, scene: &mut Scene) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                ($($name.draw(scene),)*);
            }
        }

        impl<$($name: IntoView),*> Mountable for ($($name,)*) {
            fn build(self, cx: &mut Context) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                ($($name.build(cx),)*);
            }
        }
    };
}

impl_tuple_macro!(
    view_tuple,
    A, B, C, D, E,
    F, G, H, I, J,
    K, L, M, N, O,
    P, Q, R, S, T,
    U, V, W, X, Y,
    Z
);

#[cfg(test)]
mod view_tuple_test {
    use super::*;
    use crate::widget::*;

    #[test]
    fn children() {
        let children = Children((
            h_stack((button("+", || {}), circle())),
            circle(),
        ));

        children.0.for_each(|widget| println!("{}", widget.debug_name()));

        assert!(children.debug_name().contains(children.0.debug_name()));
    }
}
