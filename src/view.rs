use crate::widget::{Widget, WidgetId, Children};
use crate::context::Context;
use crate::layout::LayoutRules;

/// wrapper over [`Widget`] trait to be stored inside [`ViewStorage`]
pub struct View {
    pub(crate) widget: Box<dyn Widget>,
}

impl View {
    fn new(widget: impl IntoView + 'static) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }

    fn build(self, cx: &mut Context) {
        let current = cx.current.take();
        let id = cx.create_id();
        let state = self.widget.state();

        cx.layout.tree.insert(id, current);
        cx.state.insert_state(&id, state.clone());

        if let Some(children) = self.widget.children() {
            cx.current = Some(id);
            cx.layout.rules.insert(id, LayoutRules::default());

            children.drain().for_each(|child| {
                let child_view = Self { widget: child };
                child_view.build(cx);
            });
        }

        cx.current = current;
    }

    // pub(crate) fn detect_hover(&self, rects: &[Rect], cursor: &mut Cursor) {
    //     if let Some(id) = cursor.hover.curr {
    //         let rect = &rects[id.index()];
    //         let contains = rect.contains(cursor.hover_pos());

    //         if !contains {
    //             cursor.hover.curr = self.find_parent(&id).map(|parent| parent.id());
    //         } else {
    //             let this = self.find_visible(&id).unwrap();
    //             cursor.hover.curr = this.children().unwrap()
    //                 .iter_visible()
    //                 .find_map(|child| {
    //                     let child_rect = &rects[child.id().index()];
    //                     child_rect.contains(cursor.hover_pos())
    //                         .then_some(child.id())
    //                 })
    //                 .or(Some(id));
    //         }
    //     } else {
    //         let mut current = &self.widget;

    //         while let Some(children) = current.children() {
    //             if let Some(hovered) = children.iter_visible()
    //                 .find_map(|child| {
    //                     let index = child.id().index();
    //                     rects[index].contains(cursor.hover_pos())
    //                         .then_some(child)
    //                 }) {
    //                 current = hovered
    //             } else {
    //                 break
    //             }
    //         }

    //         cursor.hover.curr = current
    //             .state()
    //             .flag
    //             .is_hoverable()
    //             .then_some(current.id());
    //     }
    // }

    // pub(crate) fn find_parent(&self, id: &WidgetId) -> Option<&dyn IntoView> {
    //     if self.widget.as_ref().id() == id { return None }

    //     let mut current = self.widget.as_ref();

    //     while let Some(children) = current.children() {
    //         if children.iter_all().any(|child| child.id() == id) { break }

    //         children
    //             .iter_all()
    //             .for_each(|child| current = child);
    //     }

    //     current
    //         .id()
    //         .ne(id)
    //         .then_some(current)
    // }

    // pub(crate) fn find_visible(&self, id: &WidgetId) -> Option<&dyn IntoView> {
    //     let mut current = self.widget.as_ref();

    //     while let Some(children) = current.children() {
    //         if current.id() == id { break }

    //         children
    //             .iter_visible()
    //             .for_each(|child| current = child.as_ref());
    //     }

    //     Some(current)
    // }

    // fn insert<T: Widget + 'static>(&mut self, parent: &WidgetId, widget: T) {
    //     if let Some(mut p) = self.find_mut(parent)
    //         && let Some(vec) = p.children_mut()
    //     {
    //         vec.push(Box::new(widget));
    //     }
    // }

    // fn remove(&mut self, id: &WidgetId) -> Option<Box<dyn Widget>> {
    //     self.parent_mut(id)
    //         .and_then(|mut parent| {
    //             parent.children_mut()
    //                 .and_then(|children| {
    //                     children.iter()
    //                         .position(|w| w.id() == id)
    //                         .map(|index| children.remove(index))
    //                 })
    //         })
    // }
}

pub trait IntoView: Widget {
    fn into_view(self) -> View;
}

impl<T: Widget + 'static> IntoView for T {
    fn into_view(self) -> View {
        View::new(self)
    }
}

impl std::fmt::Debug for Box<dyn IntoView> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl std::fmt::Debug for &dyn IntoView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .field("children", &self.children().unwrap_or(&Children::new()))
            .finish()
    }
}

impl std::fmt::Debug for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.widget.fmt(f)
    }
}
