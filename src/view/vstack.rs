use util::Size;

use crate::callback::CALLBACKS;
use crate::context::{Alignment, LayoutCtx};
use crate::{Pixel, Rgba};
use crate::shapes::{Attributes, Shape, ShapeKind};

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

    // pub fn on_hover<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
    //     self
    // }

    // pub fn on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
    //     self
    // }

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for VStack {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[AnyView]> { Some(&self.children) }

    fn shape(&self) -> Shape {
        Shape::filled(Rgba::DARK_GRAY, ShapeKind::Rect)
    }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut LayoutCtx, attr: &mut Attributes) {
        cx.align_vertically();
        cx.assign_position(attr);
    }

    fn attribs(&self) -> Attributes {
        let mut size = Size::new(0, 0);
        if !self.children.is_empty() {
            self.children.iter().for_each(|child| {
                let child_attr = child.attribs();
                let child_size = child_attr.dims;
                size.height += child_size.height;
                size.width = size.width.max(child_size.width + self.padding() * 2);
            });
            let child_len = self.children.len() as u32;
            size.height += self.padding() * 2 + self.spacing() * (child_len - 1);
        } else { size = (1, 1).into() }
        Attributes::new(size)
    }

    fn padding(&self) -> u32 { 20 }

    fn spacing(&self) -> u32 { 20 }

    fn alignment(&self) -> Alignment { Alignment::Vertical }
}

impl IntoView for VStack {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
