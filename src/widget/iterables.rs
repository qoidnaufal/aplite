use aplite_types::Rect;

use crate::view::IntoView;
use crate::widget::Widget;
use crate::layout::Axis;
use crate::context::{BuildCx, CursorCx, LayoutCx};

fn build<'a, T: Widget>(name: &'a [T], cx: &mut BuildCx<'_>) -> bool {
    let mut path_id = cx.pop();

    let dirty = name.iter().fold(false, |dirty, widget| {
        let content_dirty = cx.with_id(path_id, |cx| widget.build(cx));
        path_id += 1;
        dirty || content_dirty
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

fn detect_hover<'a, T: Widget>(name: &'a [T], cx: &mut CursorCx<'_>) -> bool {
    let mut id_path = cx.pop();

    let hovered = name.iter().any(|widget| {
        let content_hovered = cx.with_id(id_path, |cx| widget.detect_hover(cx));
        id_path += 1;
        content_hovered
    });

    cx.push(id_path);

    hovered
}

impl<T: Widget> Widget for Vec<T> {
    fn build(&self, cx: &mut BuildCx<'_>) -> bool {
        build(self, cx)
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        layout(self, cx);
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        detect_hover(self, cx)
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
        build(self, cx)
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        layout(self, cx);
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        detect_hover(self, cx)
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
        build(self, cx)
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        layout(self, cx);
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        detect_hover(self, cx)
    }
}

impl<T: Widget + 'static, const N: usize> IntoView for [T; N] {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}
