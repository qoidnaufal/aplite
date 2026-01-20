use std::ptr::NonNull;

use aplite_types::Rect;

use crate::layout::Axis;
use crate::widget::Widget;
use crate::context::{BuildCx, CursorCx, LayoutCx};

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
pub trait IntoView: Sized + 'static {
    type View: Widget;
    fn into_view(self) -> Self::View;
}

// impl<IV: Widget + Sized + 'static> IntoView for IV {
//     type View = IV;

//     fn into_view(self) -> Self::View {
//         self
//     }
// }

/*
#########################################################
#
# AnyView
#
#########################################################
*/

pub trait ToAnyView: IntoView {
    fn into_any(self) -> AnyView {
        AnyView::new(self.into_view())
    }
}

impl<IV: IntoView> ToAnyView for IV {}

pub struct AnyView {
    widget: NonNull<()>,
    drop_fn: Option<unsafe fn(NonNull<()>)>,
}

impl Drop for AnyView {
    fn drop(&mut self) {
        let drop_fn = self.drop_fn.take();
        if let Some(drop_fn) = drop_fn {
            unsafe { drop_fn(self.widget) }
        }
        self.drop_fn = drop_fn;
    }
}

impl AnyView {
    pub(crate) fn new<W: Widget + Sized>(widget: W) -> Self {
        #[inline]
        unsafe fn drop_fn<W>(ptr: NonNull<()>) {
            unsafe {
                let _ = Box::from_raw(ptr.cast::<W>().as_ptr());
            }
        }

        Self {
            widget: NonNull::from_mut(Box::leak(Box::new(widget))).cast(),
            drop_fn: Some(drop_fn::<W>)
        }
    }

    pub fn as_ref<'a>(&'a self) -> &'a dyn Widget {
        unsafe {
            let ptr = self.widget.as_ptr() as *const dyn Widget;
            &*ptr
        }
    }

    pub fn as_mut<'a>(&'a mut self) -> &'a mut dyn Widget {
        unsafe {
            let ptr = self.widget.as_ptr() as *mut dyn Widget;
            &mut *ptr
        }
    }
}

impl Widget for AnyView {
    fn build(&self, cx: &mut BuildCx<'_>) {
        self.as_ref().build(cx);
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.as_ref().layout(cx);
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        self.as_ref().detect_hover(cx)
    }
}

/*
#########################################################
#
# ViewTuple
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
        impl<$($name: Widget),*> Widget for ($($name,)*) {
            fn build(&self, cx: &mut BuildCx<'_>) {
                let mut path_id = cx.pop();

                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                ($(
                    cx.with_id(path_id, |cx| {
                        $name.build(cx);
                        path_id += 1;
                    }),
                )*);

                cx.push(path_id);
            }

            fn layout(&self, cx: &mut LayoutCx<'_>) {
                #[allow(non_snake_case)]
                fn for_each<$($name: Widget),*>(
                    $($name: &$name,)*
                    mut f: impl FnMut(&dyn Widget)
                ) {
                    ($(f($name),)*);
                }

                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                let mut count = 0;

                for_each($($name,)* |_| count += 1);

                let mut path_id = cx.pop();

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

                let mut cx = LayoutCx::new(cx.cx, cx.rules, bound);

                for_each($($name,)* |w| cx.with_id(path_id, |cx| {
                    w.layout(cx);
                    path_id += 1;
                }));

                cx.push(path_id);
            }

            fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
                #[allow(non_snake_case)]
                fn any<$($name: Widget),*>(
                    $($name: &$name,)*
                    mut f: impl FnMut(&dyn Widget) -> bool
                ) -> bool {
                    ($(
                        if f($name) {
                            return true;
                        }
                    ,)*);

                    false
                }

                let mut path_id = cx.pop();

                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                let res = any($($name,)* |w| cx.with_id(path_id, |cx| {
                    let res = w.detect_hover(cx);
                    path_id += 1;
                    res
                }));

                cx.push(path_id);

                res
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
    use crate::layout::{Padding, Spacing};
    use crate::context::Context;
    use crate::widget::*;
    use aplite_reactive::*;
    use aplite_types::Length;

    #[test]
    fn view_fn() {
        let (name, set_name) = Signal::split("Balo");
        let view = move || name;
        let view = view.into_view();

        println!("{}", view.with_untracked(|v| v.get()));
        set_name.set("Nunez");
        println!("{}", view.with_untracked(|v| v.get()));
    }

    #[test]
    fn stack_content() {
        let s_vec = hstack(vec![
            circle.into_any(),
            button("", || {}).into_any(),
            circle().into_any(),
        ]);

        let s_tuple = hstack((
            circle,
            button("", || {}),
            circle(),
        ));

        let s_arr = hstack([
            circle.into_any(),
            button("", || {}).into_any(),
            circle().into_any(),
        ]);

        assert_eq!(s_vec.type_id(), TypeId::of::<Stack<Vec<AnyView>, Horizontal>>());
        assert_eq!(s_arr.type_id(), TypeId::of::<Stack<[AnyView; 3], Horizontal>>());
        assert_ne!(s_vec.type_id(), s_arr.type_id());
        assert_ne!(s_tuple.type_id(), s_arr.type_id());
    }

    #[test]
    fn either_test() {
        let (when, set_when) = Signal::split(false);

        let e = either(
            move || when.get(),
            move || hstack((circle, button("", || {}))),
            || button("+", || {}),
        );

        let widget = e.into_view();

        let name = widget.debug_name();
        println!("{name}");
        assert!(name.contains("Button"));

        println!();
        set_when.set(true);

        let name = widget.debug_name();
        println!("{name}");
        assert!(name.contains("Stack"));
    }

    fn view(when: SignalRead<bool>) -> impl IntoView {
        vstack((
            circle().style(|state| state.radius = Length::Fixed(20.)),
            either(
                move || when.get(),
                || button((69, ""), || {}),
                circle,
            ),
        ))
        .style(|state| {
            state.padding = Padding::splat(5);
            state.spacing = Spacing(5);
        })
    }

    #[test]
    fn build_and_layout() {
        let mut cx = Context::new((500, 500).into());
        let (signal, set_signal) = Signal::split(false);
        let view = view(signal).into_view();

        cx.build(&view);
        cx.layout(&view);

        println!("{:?}", cx.layout_nodes);
        println!("{:?}\n", cx.elements);

        set_signal.set(true);

        cx.build(&view);
        cx.layout(&view);

        println!("{:?}", cx.layout_nodes);
        println!("{:?}\n", cx.elements);

        set_signal.set(false);

        cx.build(&view);
        cx.layout(&view);

        println!("{:?}", cx.layout_nodes);
        println!("{:?}\n", cx.elements);
    }
}
