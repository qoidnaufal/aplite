use crate::color::{Pixel, Rgba};
use crate::properties::{Properties, Shape};
use crate::context::Context;

use super::{NodeId, Render, View};

pub fn stack(cx: &mut Context, child_view: impl IntoIterator<Item = View>) -> Stack {
    Stack::new(cx, child_view)
}

pub struct Stack {
    id: NodeId,
    children: Option<Vec<View>>,
    properties: Properties,
}

impl Stack {
    pub fn new(cx: &mut Context, child_view: impl IntoIterator<Item = View>) -> Self {
        Self::init().child(child_view)
    }

    fn init() -> Self {
        let id = NodeId::new();
        let children = None;
        let properties = Properties::new(Rgba::DARK_GRAY, (1, 1), Shape::Rect, false);
        Self { id, children, properties }
    }

    fn child(mut self, child_view: impl IntoIterator<Item = View>) -> Self {
        self.children = Some(child_view.into_iter().collect());
        self
    }

    pub fn style<F: FnMut(&mut Properties)>(mut self, mut f: F) -> Self {
        f(&mut self.properties);
        self
    }
}

impl Render for Stack {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[View]> { self.children.as_ref().map(|cv| cv.as_slice()) }

    fn pixel(&self) -> Option<Pixel<u8>> { None }

    fn properties(&self) -> &Properties { &self.properties }
}
