use crate::callback::CALLBACKS;
use crate::context::{Alignment, LayoutCtx};
use crate::{Pixel, Rgba};
use crate::shapes::{Attributes, Shape, ShapeKind, Style};
use super::{AnyView, IntoView, NodeId, View};

pub fn hstack(child_nodes: impl IntoIterator<Item = AnyView>) -> HStack {
    HStack::new(child_nodes)
}

pub struct HStack {
    id: NodeId,
    children: Vec<AnyView>,
    style: Style,
}

impl HStack {
    fn new(child_nodes: impl IntoIterator<Item = AnyView>) -> Self {
        let id = NodeId::new();
        let children = child_nodes.into_iter().collect();
        let style = Style::new(Rgba::YELLOW, (0, 0), ShapeKind::Rect);
        Self { id, children, style }
    }

    pub fn style<F: FnMut(&mut Style)>(mut self, mut f: F) -> Self {
        f(&mut self.style);
        self
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

impl View for HStack {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[AnyView]> { Some(&self.children) }

    fn shape(&self) -> Shape { Shape::filled(&self.style) }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut LayoutCtx, attr: &mut Attributes) {
        cx.align_horizontally();
        cx.assign_position(attr);
    }

    fn attribs(&self) -> Attributes {
        let mut size = self.style.get_dimensions();
        if !self.children.is_empty() {
            self.children.iter().for_each(|child| {
                let child_attr = child.attribs();
                let child_size = child_attr.dims;
                size.width += child_size.width - 1;
                size.height = size.height.max(child_size.height + self.padding() * 2);
            });
            let child_len = self.children.len() as u32;
            size.width += self.padding() * 2 + self.spacing() * (child_len - 1);
        } else { size = (1, 1).into() }
        Attributes::new(size)
    }

    fn padding(&self) -> u32 { 20 }

    fn spacing(&self) -> u32 { 20 }

    fn alignment(&self) -> Alignment { Alignment::Horizontal }
}

impl IntoView for HStack {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
