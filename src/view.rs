use aplite_renderer::Scene;
use aplite_storage::{Component, make_component};
use aplite_types::Size;

use crate::context::Context;
use crate::layout::{LayoutCx, Axis};
use crate::widget::Widget;

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
pub trait IntoView: Widget + ForEachView + Sized + 'static {
    type View: IntoView;
    /// View basically is just a build context for the widget which implements it.
    /// Internally it's a `Box<dyn FnOnce(&mut ViewStorage) -> Entity + 'a>`
    fn into_view(self) -> Self::View;
}


/*
#########################################################
#
# AnyView
#
#########################################################
*/

pub trait ToAnyView: IntoView + Sized {
    fn into_any(self) -> AnyView {
        AnyView::new(Box::new(self.into_view()))
    }
}

impl<IV: IntoView> ToAnyView for IV {}

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
    fn layout(&mut self, cx: &mut LayoutCx<'_>) {
        self.as_mut().layout(cx);
    }

    fn draw(&self, scene: &mut aplite_renderer::Scene) {
        self.as_ref().draw(scene);
    }
}

impl ForEachView for AnyView {
    fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
        f(self.as_ref())
    }

    fn for_each_mut(&mut self, mut f: impl FnMut(&mut dyn Widget)) {
        f(self.as_mut())
    }
}

impl IntoView for AnyView {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

make_component!(AnyView);

/*
#########################################################
#
# ViewTuple
#
#########################################################
*/

pub trait ForEachView: Widget + Sized {
    fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
        f(self);
    }

    fn for_each_mut(&mut self, mut f: impl FnMut(&mut dyn Widget)) {
        f(self)
    }
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
        impl<$($name: IntoView),*> ForEachView for ($($name,)*) {
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
            fn layout_node_size(&self, axis: Axis) -> Size {
                let mut s = Size::default();

                match axis {
                    Axis::Horizontal => {
                        self.for_each(|w| {
                            let content_size = w.layout_node_size(axis);
                            s.width += content_size.width;
                            s.height = s.height.max(content_size.height);
                        })
                    },
                    Axis::Vertical => {
                        self.for_each(|w| {
                            let content_size = w.layout_node_size(axis);
                            s.height += content_size.height;
                            s.width = s.width.max(content_size.width);
                        })
                    },
                }

                s
            }

            fn layout(&mut self, cx: &mut LayoutCx<'_>) {
                self.for_each_mut(|w| w.layout(cx));
            }

            fn draw(&self, scene: &mut Scene) {
                self.for_each(|w| w.draw(scene));
            }
        }

        impl<$($name: IntoView),*> IntoView for ($($name,)*) {
            type View = ($($name::View,)*);

            fn into_view(self) -> Self::View {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                ($($name.into_view(),)*)
            }
        }

        // impl<$($name: IntoView),*> IntoView for ($($name,)*) {
        //     type View = Vec<AnyView>;

        //     fn into_view(self) -> Self::View {
        //         #[allow(non_snake_case)]
        //         let ($($name,)*) = self;

        //         let mut vec = vec![];
        //         ($(vec.push($name.into_any()),)*);

        //         vec
        //     }
        // }
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
mod view_test {
    use std::any::{TypeId, Any};
    use super::*;
    use crate::widget::*;
    use aplite_reactive::*;

    #[test]
    fn view_fn() {
        let name = Signal::new("Balo");

        let view = move || name.get();

        let debug_name = view.debug_name();
        println!("{debug_name}");
        assert!(debug_name.contains("&str"));
    }

    #[test]
    fn stack_content() {
        let v0 = vstack(circle());
        let v0 = v0.into_view();
        assert_eq!(v0.type_id(), TypeId::of::<Stack<CircleWidget, Vertical>>());

        let hv = hstack(vec![
            circle().into_any(),
            button("", || {}).into_any(),
            circle().into_any(),
        ]);
        let ht = hstack((
            circle(),
            button("", || {}),
            circle(),
        ));

        assert!(size_of_val(&hv) < size_of_val(&ht));
        assert_eq!(hv.type_id(), TypeId::of::<Stack<Vec<AnyView>, Horizontal>>());
        assert_ne!(hv.type_id(), ht.type_id());

        let vf = vstack(circle);
        let tid_vf = vf.type_id();
        let iv = vf.into_view();

        assert_ne!(tid_vf, iv.type_id());
        assert_eq!(iv.type_id(), v0.type_id());
    }

    #[test]
    fn either_test() {
        let (when, set_when) = Signal::split(false);

        let e = either(
            move || when.get(),
            move || hstack((circle, button("", || {}))),
            || button("+", || {}),
        );

        let name = e.debug_name();
        println!("{name}");
        assert!(name.contains("Button"));

        println!();
        set_when.set(true);

        let name = e.debug_name();
        println!("{name}");
        assert!(name.contains("Stack"));
    }

    #[test]
    fn for_each_view() {
        let ht = hstack((
            circle(),
            button("", || {}),
        ));

        ht.for_each(|w| {
            println!("{}", w.debug_name())
        });

        circle.for_each(|w| println!("{}", w.debug_name()));
    }
}
