mod button;
mod image;
mod vstack;
mod hstack;

pub use {
    button::*,
    image::*,
    vstack::*,
    hstack::*,
};
use std::sync::atomic::{AtomicU64, Ordering};
use crate::{
    callback::CALLBACKS, color::Rgb, context::LayoutCtx, shapes::{Shape, ShapeKind}, storage::WidgetStorage
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    fn new() -> Self {
        static NODE_ID: AtomicU64 = AtomicU64::new(0);
        Self(NODE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub type AnyView = Box<dyn View>;

pub trait View {
    fn id(&self) -> NodeId;
    fn shape(&self) -> Shape;
    fn children(&self) -> Option<&[AnyView]>;
    fn layout(&self, cx: &mut LayoutCtx);
    fn insert_into(&self, storage: &mut WidgetStorage) {
        let id = self.id();
        let mut shape = self.shape();
        if storage.layout.get_parent(&id).is_none() {
            shape.color = Rgb::BLACK;
            shape.cached_color.replace(Rgb::BLACK);
        }
        storage.nodes.push(id);
        storage.shapes.insert(id, shape);
        if let Some(children) = self.children() {
            children.iter().for_each(|child| {
                child.insert_into(storage);
            });
        }
    }
}

pub trait IntoView: Sized {
    type V: View + 'static;
    fn into_view(self) -> Self::V;
    fn into_any(self) -> AnyView { Box::new(self.into_view()) }
}

pub struct DynView(AnyView);

impl View for DynView {
    fn id(&self) -> NodeId {
        self.0.id()
    }

    fn shape(&self) -> Shape {
        self.0.shape()
    }

    fn children(&self) -> Option<&[AnyView]> {
        self.0.children()
    }

    fn layout(&self, cx: &mut LayoutCtx) {
        self.0.layout(cx);
    }
}

impl<F, IV> IntoView for F
where
    F: Fn() -> IV + 'static,
    IV: IntoView + 'static
{
    type V = DynView;
    fn into_view(self) -> Self::V {
        let a = self().into_any();
        DynView(a)
    }
}

pub struct TestTriangleWidget {
    id: NodeId,
}

impl TestTriangleWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::filled(Rgb::RED, ShapeKind::FilledTriangle, (500, 500))
    }

    pub fn on_hover<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    pub fn on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for TestTriangleWidget {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn children(&self) -> Option<&[Box<dyn View>]> {
        None
    }

    fn shape(&self) -> Shape {
        self.shape()
    }

    fn layout(&self, layout: &mut LayoutCtx) {
        if layout.get_parent(&self.id()).is_some() {
            let next_pos = layout.next_child_pos();
            layout.insert_pos(self.id(), next_pos);
        } else {
            let next_pos = layout.next_pos();
            layout.insert_pos(self.id(), next_pos);
        }
    }
}

impl IntoView for TestTriangleWidget {
    type V = Self;
    fn into_view(self) -> Self::V {
        self
    }
}
