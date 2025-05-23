use std::marker::PhantomData;
use aplite_types::Rgba;
use aplite_renderer::Shape;

mod button;
mod image;
mod stack;

use crate::context::Context;
use crate::context::properties::Properties;
use crate::context::tree::NodeId;

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
        cx.insert(node_id, parent, self.properties().with_name(self.debug_name()));
        cx.set_current_entity(Some(node_id));
        child_fn(cx);
        cx.set_current_entity(parent);

        View::new(node_id, cx)
    }
}

pub struct View<'a, IV: IntoView> {
    entity: NodeId,
    cx: &'a mut Context,
    inner: PhantomData<IV>,
}

impl<'a, IV: IntoView> View<'a, IV> {
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

    pub fn style<F: Fn(&mut Properties) + 'static>(self, style_fn: F) -> Self {
        let prop = self.cx.get_node_data_mut(&self.id());
        style_fn(prop);
        self.cx.add_style_fn(self.id(), style_fn);
        self
    }
}

#[derive(Clone)]
pub struct TestCircleWidget {
    properties: Properties,
}

impl TestCircleWidget {
    pub fn new(cx: &mut Context) -> View<Self> {
        let properties = Properties::new()
            .with_size((100, 100))
            .with_shape(Shape::Circle)
            .with_fill_color(Rgba::RED);
        Self { properties }.into_view(cx, |_| {})
    }
}

impl IntoView for TestCircleWidget {
    fn debug_name(&self) -> Option<&'static str> { Some("TestCircleWidget") }
    fn properties(&self) -> Properties { self.properties }
}

// #[derive(Clone)]
// pub struct TestTriangleWidget {
//     properties: Properties,
// }

// impl TestTriangleWidget {
//     pub fn new(cx: &mut Context) -> View<Self> {
//         let properties = Properties::new()
//             .with_size((300, 300))
//             .with_shape(Shape::Triangle)
//             .with_fill_color(Rgba::RED);
//         Self { properties }.into_view(cx, |_| {})
//     }
// }

// impl IntoView for TestTriangleWidget {
//     fn debug_name(&self) -> Option<&'static str> { Some("TestTriangleWidget") }
//     fn properties(&self) -> Properties { self.properties }
// }
