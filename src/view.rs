mod button;
mod image;
mod stack;

use std::marker::PhantomData;

use crate::properties::{Shape, Properties};
use crate::context::Context;
use crate::color::Rgba;
use crate::tree::NodeId;

pub use {
    button::*,
    image::*,
    stack::*,
};

pub trait IntoView: Sized {
    fn debug_name(&self) -> Option<&'static str> { None }
    fn properties(&self) -> Properties;

    fn into_view<F>(self, cx: &mut Context, child_fn: F) -> View<Self>
    where F: FnOnce(&mut Context),
    {
        let node_id = cx.create_entity();
        let parent = cx.current_entity();
        cx.insert(node_id, parent, self.properties(), self.debug_name());
        cx.set_current_entity(Some(node_id));

        child_fn(cx);

        cx.set_current_entity(parent);

        View::new(node_id, cx)
    }
}

pub struct View<'a, R: IntoView> {
    entity: NodeId,
    cx: &'a mut Context,
    inner: PhantomData<R>,
}

impl<'a, R: IntoView> View<'a, R> {
    fn new(entity: NodeId, cx: &'a mut Context) -> Self {
        Self {
            entity,
            cx,
            inner: PhantomData,
        }
    }

    pub(crate) fn id(&self) -> NodeId {
        self.entity
    }

    pub fn style<P: Fn(&mut Properties) + 'static>(self, f: P) -> Self {
        let prop = self.cx.get_node_data_mut(&self.id());
        f(prop);
        self.cx.add_style_fn(self.id(), f);
        self
    }
}

#[derive(Clone)]
pub struct TestTriangleWidget {
    properties: Properties,
}

impl TestTriangleWidget {
    pub fn new(cx: &mut Context) -> View<Self> {
        let properties = Properties::new(Rgba::RED, (300, 300), Shape::Triangle, false);
        Self { properties }.into_view(cx, |_| {})
    }
}

impl IntoView for TestTriangleWidget {
    fn debug_name(&self) -> Option<&'static str> { Some("TestTriangleWidget") }
    fn properties(&self) -> Properties { self.properties }
}

#[derive(Clone)]
pub struct TestCircleWidget {
    properties: Properties,
}

impl TestCircleWidget {
    pub fn new(cx: &mut Context) -> View<Self> {
        let properties = Properties::new(Rgba::RED, (300, 300), Shape::Circle, false);
        Self { properties }.into_view(cx, |_| {})
    }
}

impl IntoView for TestCircleWidget {
    fn debug_name(&self) -> Option<&'static str> { Some("TestCircleWidget") }
    fn properties(&self) -> Properties { self.properties }
}
