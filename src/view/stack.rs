use crate::color::{Pixel, Rgba};
use crate::properties::{Properties, Shape};
use crate::context::Context;

use super::{AnyView, IntoView, NodeId, View};

pub fn stack(child_nodes: impl IntoIterator<Item = AnyView>) -> Stack {
    Stack::new(child_nodes)
}

pub struct Stack {
    id: NodeId,
    children: Vec<Box<dyn View>>,
    inner: Properties,
}

impl Stack {
    fn new(child_nodes: impl IntoIterator<Item = AnyView>) -> Self {
        let id = NodeId::new();
        let children = child_nodes.into_iter().collect();
        let inner = Properties::new(Rgba::DARK_GRAY, (1, 1), Shape::Rect, false);
        Self { id, children, inner }
    }

    pub fn style<F: FnMut(&mut Properties)>(mut self, mut f: F) -> Self {
        f(&mut self.inner);
        self
    }
}

impl View for Stack {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[AnyView]> { Some(&self.children) }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut Context) {
        cx.set_orientation(self.id);
        cx.set_alignment(self.id);
        cx.set_spacing(self.id);
        cx.set_padding(self.id);

        cx.assign_position(self.id);
    }

    fn properties(&self) -> &Properties { &self.inner }
}

impl IntoView for Stack {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
