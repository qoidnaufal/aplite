use math::Size;

use crate::callback::CALLBACKS;
use crate::context::CONTEXT;
use crate::Rgb;
use crate::shapes::{Shape, ShapeKind};

use super::{AnyView, IntoView, NodeId, View};

pub fn vstack(child_nodes: impl IntoIterator<Item = AnyView>) -> VStack {
    VStack::new(child_nodes)
}

pub struct VStack {
    id: NodeId,
    children: Vec<Box<dyn View>>,
}

impl VStack {
    fn new(child_nodes: impl IntoIterator<Item = AnyView>) -> Self {
        let id = NodeId::new();
        let children = child_nodes.into_iter().collect();
        Self { id, children }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn padding(&self) -> u32 {
        50
    }

    fn shape(&self) -> Shape {
        let mut size = Size::new(0, 0);
        if !self.children.is_empty() {
            self.children.iter().for_each(|child| {
                let child_size = child.shape().dimensions;
                size.height += child_size.height;
                size.width = size.width.max(child_size.width + self.padding() * 2);
            });
            let child_len = self.children.len() as u32;
            size.height += self.padding() * (child_len + 1);
        } else {
            size = (1, 1).into();
        }
        Shape::filled(Rgb::GRAY, ShapeKind::FilledRectangle, size)
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

impl View for VStack {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn children(&self) -> Option<&[AnyView]> {
        Some(&self.children)
    }

    fn shape(&self) -> Shape {
        self.shape()
    }

    fn layout(&self) {
        let dimensions = self.shape().dimensions;
        CONTEXT.with_borrow_mut(|cx| {
            let used_space = cx.layout.used_space();
            if cx.layout.get_position(&self.id()).is_none() {
                cx.layout.insert(self.id(), (used_space.x, used_space.y).into());
                cx.layout.set_used_space(|space| {
                    space.y += dimensions.height;
                });
                if !self.children.is_empty() {
                    let mut child_space = used_space;
                    self.children.iter().for_each(|child| {
                        let child_shape = child.shape();
                        cx.layout.insert(child.id(), (child_space.x + self.padding(), child_space.y + self.padding()).into());
                        child_space.y += child_shape.dimensions.height + self.padding();
                    });
                }
            }
        });
    }
}

impl IntoView for VStack {
    type V = Self;
    fn into_view(self) -> Self::V {
        self
    }
}
