use crate::callback::CALLBACKS;
use crate::properties::{Shape, Properties};
use crate::color::{Pixel, Rgba};
use crate::context::Context;

use super::{AnyView, IntoView, NodeId, View};

pub fn button() -> Button { Button::new() }

pub struct Button {
    id: NodeId,
    inner: Properties,
}

impl Button {
    fn new() -> Self {
        let id = NodeId::new();
        let inner = Properties::new(Rgba::RED, (200, 50), Shape::RoundedRect, false);
        Self { id, inner }
    }

    pub fn style<F: FnOnce(&mut Properties)>(mut self, f: F) -> Self {
        f(&mut self.inner);
        self
    }

    pub fn on_click<F: Fn() + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cb| cb.insert(self.id(), f));
        self
    }
}

impl View for Button {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[AnyView]> { None }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut Context) { cx.assign_position(self.id) }

    fn properties(&self) -> &Properties { &self.inner }
}

impl IntoView for Button {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
