use aplite_storage::{Tree, Entity};

use crate::widget::{Widget, WidgetId, Children};
use crate::cursor::Cursor;
use crate::context::Context;
use crate::layout::State;

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

    pub(crate) fn id(&self) -> WidgetId {
        self.widget.id()
    }

    // pub(crate) fn detect_hover(&self, cursor: &Cursor) -> Option<&Box<dyn Widget>> {
    //     let mut current = &self.widget;

    //     while let Some(children) = current.children_ref() {
    //         if let Some(hovered) = children.visible_boxed()
    //             .find_map(|child| {
    //                 child.node_ref()
    //                     .unwrap()
    //                     .upgrade()
    //                     .borrow()
    //                     .rect
    //                     .contains(cursor.hover.pos)
    //                     .then_some(child)
    //             }) {
    //             current = hovered
    //         } else {
    //             break
    //         }
    //     }

    //     current
    //         .node()
    //         .is_hoverable()
    //         .then_some(current)
    // }

    // pub(crate) fn find_parent(&self, id: &WidgetId) -> Option<&Box<dyn Widget>> {
    //     if self.widget.as_ref().id() == id { return None }

    //     let mut current = &self.widget;

    //     while let Some(children) = current.children_ref() {
    //         if children.iter().any(|child| child.id() == id) { break }

    //         children
    //             .all_boxed()
    //             .for_each(|child| current = child);
    //     }

    //     current
    //         .id()
    //         .ne(id)
    //         .then_some(current)
    // }

    // pub(crate) fn find_visible(&self, id: &WidgetId) -> Option<&dyn Widget> {
    //     let mut current = self.widget.as_ref();

    //     while let Some(children) = current.children_ref() {
    //         if current.id() == id { break }

    //         children
    //             .visible_ref()
    //             .for_each(|child| current = child);
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
            .field("id", &self.id())
            .field("children", &self.children().unwrap_or(&Children::new()))
            .finish()
    }
}
