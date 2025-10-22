use aplite_storage::{Arena, ArenaItem, Array};

use crate::widget::{Widget, WidgetId};
use crate::context::Context;
use crate::cursor::Cursor;

pub trait IntoView: Widget {
    fn into_view(self) -> View;
}

/// wrapper over [`Widget`] trait to be stored inside [`ViewStorage`]
pub struct View {
    pub(crate) widget: Box<dyn Widget>,
}

impl View {
    pub fn new<T: IntoView + 'static>(widget: T) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }

    pub(crate) fn detect_hover(&self, cursor: &mut Cursor) {
    }
}


impl std::fmt::Debug for Box<dyn IntoView> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl std::fmt::Debug for &dyn IntoView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scoped_name = std::any::type_name::<Self>();
        let name = scoped_name.split("::").last().unwrap_or(scoped_name);
        f.debug_struct(name)
            .finish()
    }
}

impl std::fmt::Debug for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.widget.fmt(f)
    }
}
