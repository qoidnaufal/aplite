use crate::layout::LayoutCx;
use crate::widget::Widget;
use crate::context::{BuildCx, Context};

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
    fn into_view(self) -> Self::View;
}

impl<IV: Widget + Sized + 'static> IntoView for IV {
    type View = IV;

    fn into_view(self) -> Self::View {
        self
    }
}

// pub trait ForEachView: IntoView {
//     fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
//         f(self);
//     }

//     fn for_each_mut(&mut self, mut f: impl FnMut(&mut dyn Widget)) {
//         f(self)
//     }

//     fn count(&self) -> usize {
//         let mut count = 0;
//         self.for_each(|_| count += 1);
//         count
//     }
// }

// pub struct Children<FE: ForEachView>(FE);

/*
#########################################################
#
# AnyView
#
#########################################################
*/

pub trait ToAnyView: IntoView {
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

    pub fn as_ref<'a>(&'a self) -> &'a dyn Widget {
        self.widget.as_ref()
    }

    pub fn as_mut<'a>(&'a mut self) -> &'a mut dyn Widget {
        self.widget.as_mut()
    }
}

impl Widget for AnyView {
    fn build(&self, cx: &mut BuildCx<'_>) {
        self.widget.build(cx);
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        self.widget.layout(cx);
    }

    fn detect_hover(&self, cx: &mut Context) {
        let rect = cx.get_layout_node().unwrap();
        if rect.contains(&cx.cursor.hover.pos) {}
    }
}

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
        impl<$($name: IntoView),*> Widget for ($($name,)*) {
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
                let mut path_id = cx.pop();

                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                ($(
                    cx.with_id(path_id, |cx| {
                        $name.layout(cx);
                        path_id += 1;
                    }),
                )*);

                cx.push(path_id);
            }

            fn detect_hover(&self, cx: &mut Context) {
                let mut path_id = cx.pop();

                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                ($(
                    cx.with_id(path_id, |cx| {
                        $name.detect_hover(cx);
                        path_id += 1;
                    }),
                )*);

                cx.push(path_id);
            }
        }

        // impl<$($name),*> ForEachView for ($($name,)*)
        // where
        //     // ($($name,)*): IntoView,
        //     $($name: IntoView),*,
        // {
        //     fn for_each(&self, mut f: impl FnMut(&dyn Widget)) {
        //         #[allow(non_snake_case)]
        //         let ($($name,)*) = self;

        //         ($(f($name),)*);
        //     }

        //     fn for_each_mut(&mut self, mut f: impl FnMut(&mut dyn Widget)) {
        //         #[allow(non_snake_case)]
        //         let ($($name,)*) = self;

        //         ($(f($name),)*);
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
    use crate::layout::{Padding, Spacing};
    use crate::context::Context;
    use crate::widget::*;
    use aplite_reactive::*;
    use aplite_types::Length;

    #[test]
    fn view_fn() {
        let name = Signal::new("Balo");
        let view = move || name;

        let debug_name = view.debug_name();
        println!("{debug_name}");
        assert_eq!(debug_name, "&str");
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

        let name = e.debug_name();
        println!("{name}");
        assert!(name.contains("Button"));

        println!();
        set_when.set(true);

        let name = e.debug_name();
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
        let view = view(signal);

        cx.build(&view);
        cx.layout(&view);

        println!("{:?}", cx.layout_nodes);
        println!("{:?}\n", cx.view_ids);

        set_signal.set(true);

        cx.build(&view);
        cx.layout(&view);

        println!("{:?}", cx.layout_nodes);
        println!("{:?}\n", cx.view_ids);

        set_signal.set(false);

        cx.build(&view);
        cx.layout(&view);

        println!("{:?}", cx.layout_nodes);
        println!("{:?}\n", cx.view_ids);
    }
}
