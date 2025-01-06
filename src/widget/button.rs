use crate::{color::Rgb, shapes::{Shape, ShapeType}, types::{Size, Vector2}};

use super::{CallBack, NodeId, Widget, CALLBACKS};

#[derive(Debug, Clone, Copy)]
pub struct Button {
    id: NodeId,
}

impl Button {
    pub fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::new(Vector2::new(), Size::new(500, 500), Rgb::RED, ShapeType::Rectangle)
    }

    pub fn on_click<F: FnMut() + 'static>(&self, mut f: F) -> Self {
        let a = &mut f as *mut dyn FnMut();
        CALLBACKS.with_borrow_mut(|cbs| cbs.insert(self.id(), CallBack(a)));
        *self
    }
}

impl Widget for Button {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn shape(&self) -> Shape {
        self.shape()
    }
}
