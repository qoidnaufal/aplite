mod button;
mod image;
mod vstack;
mod hstack;

use math::Vector2;

pub use {
    button::*,
    image::*,
    vstack::*,
    hstack::*,
};
use std::sync::atomic::{AtomicU64, Ordering};
use crate::{
    callback::CALLBACKS, color::Rgb, context::CONTEXT, shapes::{Shape, ShapeKind}
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
    fn layout(&self);
}

pub trait IntoView: Sized {
    type V: View + 'static;
    fn into_view(self) -> Self::V;
    fn into_any(self) -> AnyView { Box::new(self.into_view()) }
}

pub struct TestTirangleWidget {
    id: NodeId,
}

impl TestTirangleWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::filled(Rgb::BLACK, ShapeKind::FilledTriangle, (500, 500))
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

impl View for TestTirangleWidget {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn children(&self) -> Option<&[Box<dyn View>]> {
        None
    }

    fn shape(&self) -> Shape {
        self.shape()
    }

    fn layout(&self) {
        let dimensions = self.shape().dimensions;
        CONTEXT.with_borrow_mut(|cx| {
            if cx.layout.get_position(&self.id()).is_none() {
                let used_space = cx.layout.used_space();
                cx.layout.insert(self.id(), (0, used_space.y).into());
                cx.layout.set_used_space(|space| space.y += dimensions.height);
            }
        });
    }
}

impl IntoView for TestTirangleWidget {
    type V = Self;
    fn into_view(self) -> Self::V {
        self
    }
}
