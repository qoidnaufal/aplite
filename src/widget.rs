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
    pub static CALLBACKS: RefCell<Callbacks> = RefCell::new(Callbacks::default());
}

#[derive(Default)]
pub struct Callbacks {
    pub on_click: HashMap<NodeId, Callback>,
    pub on_drag: HashMap<NodeId, Callback>,
}

// pub struct Callback(*mut dyn FnMut(&mut Shape));
pub struct Callback(Box<dyn FnMut(&mut Shape) + 'static>);

impl std::fmt::Debug for Callback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ptr = &self.0 as *const dyn FnMut(&mut Shape);
        write!(f, "{:?}", ptr)
    }
}

impl<F: FnMut(&mut Shape) + 'static> From<F> for Callback {
    fn from(callback: F) -> Self {
        Self(Box::new(callback))
    }
}

impl std::ops::Deref for Callback {
    type Target = Box<dyn FnMut(&mut Shape) + 'static>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Callback {
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

    pub fn on_click<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> Self {
        let callback = Callback::from(f);
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), callback));
        *self
    }

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), Callback::from(f)));
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
