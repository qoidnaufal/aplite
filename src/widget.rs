mod button;
mod image;

pub use {
    button::Button,
    image::Image,
};

use std::{cell::RefCell, collections::HashMap, sync::atomic::{AtomicU64, Ordering}};

use crate::{
    color::Rgb,
    shapes::{Shape, ShapeType},
    types::{Size, Vector2}
};

thread_local! {
    pub static NODE_ID: AtomicU64 = const { AtomicU64::new(0) };
    pub static CALLBACKS: RefCell<HashMap<NodeId, CallBack>> = RefCell::new(HashMap::new());
}

pub struct CallBack(*mut dyn FnMut());

impl<F: FnMut() + 'static> From<F> for CallBack {
    fn from(mut callback: F) -> Self {
        Self(&mut callback as *mut dyn FnMut())
    }
}

impl std::ops::Deref for CallBack {
    type Target = *mut dyn FnMut();
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for CallBack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    fn new() -> Self {
        Self(NODE_ID.with(|i| i.fetch_add(1, Ordering::Relaxed)))
    }
}

pub trait Widget: std::fmt::Debug {
    fn id(&self) -> NodeId;
    fn shape(&self) -> Shape;
}

#[derive(Debug, Clone, Copy)]
pub struct TestWidget {
    id: NodeId,
}

impl TestWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::new(Vector2::new(), Size::new(500, 500), Rgb::RED, ShapeType::Triangle)
    }

    pub fn on_click<F: FnMut() + 'static>(&self, mut f: F) -> Self {
        let a = &mut f as *mut dyn FnMut();
        CALLBACKS.with_borrow_mut(|cbs| cbs.insert(self.id(), CallBack(a)));
        *self
    }
}

impl Widget for TestWidget {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn shape(&self) -> Shape {
        self.shape()
    }
}
