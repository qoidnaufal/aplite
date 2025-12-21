use std::any::type_name;

use aplite_storage::{
    Entity,
    ArenaPtr,
};

use crate::context::Context;
use crate::widget::Widget;

/*
#########################################################
#
# View
#
#########################################################
*/

pub struct View<W> {
    widget: W
}

impl<W: Widget> View<W> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
        }
    }

    pub fn as_ref(&self) -> &dyn Widget {
        &self.widget
    }

    pub fn as_mut(&mut self) -> &mut dyn Widget {
        &mut self.widget
    }
}

impl<W: Widget> std::fmt::Debug for View<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View")
            .field("kind", &type_name::<W>())
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

pub(crate) struct AnyView {
    pub(crate) ptr: ArenaPtr<dyn Widget>,
}

impl AnyView {
    pub(crate) fn new(item: ArenaPtr<dyn Widget>) -> Self {
        Self { ptr: item }
    }

    pub(crate) fn as_ref(&self) -> &dyn Widget {
        self.ptr.as_ref()
    }

    pub(crate) fn as_mut(&mut self) -> &mut dyn Widget {
        self.ptr.as_mut()
    }
}

impl std::ops::Deref for AnyView {
    type Target = dyn Widget;

    fn deref(&self) -> &Self::Target {
        std::ops::Deref::deref(&self.ptr)
    }
}

impl std::ops::DerefMut for AnyView {
    fn deref_mut(&mut self) -> &mut Self::Target {
        std::ops::DerefMut::deref_mut(&mut self.ptr)
    }
}

/*
#########################################################
#
# IntoView
#
#########################################################
*/

/// Types that automatically implement IntoView are:
/// - any type that implement Widget (`impl Widget for T`),
/// - any function that produce IntoView (`FnOnce() -> IV where IV: IntoView` or `fn() -> impl IntoView`)
pub trait IntoView: Widget + Sized + 'static {
    /// View basically is just a build context for the widget which implements it.
    /// Internally it's a `Box<dyn FnOnce(&mut ViewStorage) -> Entity + 'a>`
    fn into_view(self) -> View<Self>;
}


impl<W> IntoView for W where W: Widget + Sized + 'static {
    fn into_view(self) -> View<W> {
        View::new(self)
    }
}
