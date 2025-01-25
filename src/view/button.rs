use crate::{
    callback::CALLBACKS, color::Rgb, context::CONTEXT, shapes::{Shape, ShapeKind}
};
use super::{AnyView, IntoView, NodeId, View};

pub fn button() -> Button {
    Button::new()
}

pub struct Button {
    id: NodeId,
}

impl Button {
    fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::filled(Rgb::RED, ShapeKind::FilledRectangle, (300, 100))
    }

    pub fn on_hover<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    pub fn on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }
}

impl View for Button {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn children(&self) -> Option<&[AnyView]> {
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

impl IntoView for Button {
    type V = Self;
    fn into_view(self) -> Self::V {
        self
    }
}
