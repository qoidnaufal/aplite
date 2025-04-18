use crate::layout::{Attributes, Layout};
use crate::callback::CALLBACKS;
use crate::style::{Shape, Style};
use crate::element::Element;
use crate::color::{Pixel, Rgba};

use super::{AnyView, IntoView, NodeId, View};

pub fn button() -> Button { Button::new() }

pub struct Button {
    id: NodeId,
    style: Style,
}

impl Button {
    fn new() -> Self {
        let id = NodeId::new();
        let style = Style::new(Rgba::RED, (200, 50), Shape::RoundedRect);
        Self { id, style }
    }

    pub fn style<F: FnOnce(&mut Style)>(mut self, f: F) -> Self {
        f(&mut self.style);
        self
    }

    pub fn on_hover<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    pub fn on_click<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }

    pub fn on_drag<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for Button {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[AnyView]> { None }

    fn element(&self) -> Element {
        Element::filled(&self.style)
    }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, layout: &mut Layout) -> Attributes {
        layout.assign_position(&self.id)
    }

    fn style(&self) -> Style { self.style }
}

impl IntoView for Button {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
