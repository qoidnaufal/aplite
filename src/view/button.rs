use crate::context::{Alignment, LayoutCtx};
use crate::shapes::{Attributes, Shape, ShapeKind};
use crate::callback::CALLBACKS;
use crate::{Pixel, Rgba};
use super::{AnyView, IntoView, NodeId, View};

pub fn button() -> Button { Button::new() }

pub struct Button {
    id: NodeId,
    shape: Shape,
}

impl Button {
    fn new() -> Self {
        let id = NodeId::new();
        let shape = Shape::filled(Rgba::RED, ShapeKind::RoundedRect);
        Self { id, shape }
    }

    pub fn style<F: FnMut(&mut Shape)>(mut self, mut f: F) -> Self {
        f(&mut self.shape);
        self
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
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[AnyView]> { None }

    fn shape(&self) -> Shape {
        self.shape.clone()
    }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut LayoutCtx, attr: &mut Attributes) {
        cx.assign_position(attr);
    }

    fn attribs(&self) -> Attributes {
        Attributes::new((120, 40))
    }

    fn padding(&self) -> u32 { 0 }

    fn spacing(&self) -> u32 { 0 }

    fn alignment(&self) -> Alignment { Alignment::Vertical }
}

impl IntoView for Button {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
