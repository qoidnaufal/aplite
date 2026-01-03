use aplite_renderer::Scene;
use aplite_storage::{Component, EntityId, ComponentStorage, make_component};
use aplite_types::Size;
use aplite_bitset::Bitset;

use crate::context::Context;
use crate::layout::{LayoutCx, Axis};
use crate::widget::Widget;

/*
#########################################################
#
# Traits
#
#########################################################
*/

/// Types that automatically implement IntoView are:
/// - any type that implement Widget: `impl Widget for T`,
/// - any type that implement Mount: `impl Mount for T`,
/// - any function that produce IntoView: `FnOnce() -> IV where IV: IntoView` or `fn() -> impl IntoView`
pub trait IntoView: Widget + Sized + 'static {
    type View: Widget;
    /// View basically is just a build context for the widget which implements it.
    /// Internally it's a `Box<dyn FnOnce(&mut ViewStorage) -> Entity + 'a>`
    fn into_view(self) -> Self::View;
}

pub trait Mountable: IntoView {}

pub trait ForEachView: IntoView {
    fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
        f(self);
    }

    fn for_each_mut(&mut self, mut f: impl FnMut(&mut dyn Widget)) {
        f(self)
    }

    fn count(&self) -> usize {
        let mut count = 0;
        self.for_each(|_| count += 1);
        count
    }
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
    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.as_ref().layout(cx);
    }

    fn draw(&self, scene: &mut aplite_renderer::Scene) {
        self.as_ref().draw(scene);
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
# Macros
#
#########################################################
*/

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
        impl<$($name: Widget + 'static),*> ForEachView for ($($name,)*) {
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

        impl<$($name: Widget + 'static),*> Widget for ($($name,)*) {
            fn layout_node_size(&self, bound: Size) -> Size {
                let mut s = Size::default();
                self.for_each(|w| {
                    let cs = w.layout_node_size(bound);
                    s.width += cs.width;
                    s.height = cs.height;
                });

                s
            }

            fn layout(&self, cx: &mut LayoutCx<'_>) {
                self.for_each(|w| w.layout(cx));
            }

            fn draw(&self, scene: &mut Scene) {
                self.for_each(|w| w.draw(scene));
            }
        }

        impl<$($name: Widget + 'static),*> IntoView for ($($name,)*) {
            type View = ($($name,)*);

            fn into_view(self) -> Self::View {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                ($($name,)*)
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
        assert_eq!(debug_name, "&str");
    }

    #[test]
    fn stack_content() {
        let s_vec = hstack(vec![
            circle().into_any(),
            button("", || {}).into_any(),
            circle().into_any(),
        ]);

        let s_tuple = hstack((
            circle(),
            button("", || {}),
            circle(),
        ));

        let s_arr = hstack([
            circle().into_any(),
            button("", || {}).into_any(),
            circle().into_any(),
        ]);

        assert!(size_of_val(&s_vec) < size_of_val(&s_tuple));
        assert_eq!(s_vec.type_id(), TypeId::of::<Stack<Vec<AnyView>, Horizontal>>());
        assert_eq!(s_arr.type_id(), TypeId::of::<Stack<[AnyView; 3], Horizontal>>());
        assert_ne!(s_vec.type_id(), s_tuple.type_id());
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
            either(|| false, || button("", || {}), circle),
            vstack((circle, circle)),
            circle,
            circle(),
            button(("", circle), || {}),
        ));

        ht.for_each(|w| {
            println!("{}", w.debug_name())
        });
    }
}
