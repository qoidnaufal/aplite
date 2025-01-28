use math::Size;

use crate::callback::CALLBACKS;
use crate::context::LayoutCtx;
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

    // pub fn on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
    //     self
    // }

    // pub fn on_drag<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
    //     self
    // }
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

    fn layout(&self, cx: &mut LayoutCtx) {
        if cx.get_parent(&self.id()).is_some() {
            let next_pos = cx.next_child_pos();
            cx.insert_pos(self.id(), next_pos);
        } else {
            let next_pos = cx.next_pos();
            cx.insert_pos(self.id(), next_pos);
        }

        let current_pos = *cx.get_position(&self.id()).unwrap();
        cx.set_next_child_pos(|pos| {
            pos.x = current_pos.x + self.padding();
            pos.y = current_pos.y + self.padding();
        });

        self.children.iter().for_each(|child| {
            cx.insert_parent(child.id(), self.id());
            cx.insert_children(self.id(), child.id());
            child.layout(cx);
            cx.set_next_child_pos(|pos| {
                pos.y += child.shape().dimensions.height + self.padding();
            });
        });

        if cx.get_parent(&self.id()).is_some() {
            cx.set_next_child_pos(|pos| {
                pos.x = current_pos.x;
                pos.y = current_pos.y;
            });
        } else {
            cx.set_next_pos(|pos| {
                pos.y += self.shape().dimensions.height;
            });
        }
    }
}

impl IntoView for VStack {
    type V = Self;
    fn into_view(self) -> Self::V {
        self
    }
}
