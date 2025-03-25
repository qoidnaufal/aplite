use crate::context::{Alignment, LayoutCtx};
use crate::shapes::{Attributes, Shape, ShapeKind, Style};
use crate::callback::CALLBACKS;
use crate::{Pixel, Rgba};
use super::{AnyView, IntoView, NodeId, View};

pub fn button() -> Button { Button::new() }

pub struct Button {
    id: NodeId,
    style: Style,
}

impl Button {
    fn new() -> Self {
        let id = NodeId::new();
        let style = Style::new(Rgba::RED, (200, 50), ShapeKind::RoundedRect);
        Self { id, style }
    }

    pub fn style<F: FnMut(&mut Style)>(mut self, mut f: F) -> Self {
        f(&mut self.style);
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

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for Button {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[AnyView]> { None }

    fn shape(&self) -> Shape {
        Shape::filled(&self.style)
    }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut LayoutCtx, attr: &mut Attributes) {
        cx.assign_position(attr);
    }

    fn attribs(&self) -> Attributes {
        Attributes::new(self.style.get_dimensions())
    }

    fn padding(&self) -> u32 { 0 }

    fn spacing(&self) -> u32 { 0 }

    fn alignment(&self) -> Alignment { Alignment::Vertical }
}

impl IntoView for Button {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
