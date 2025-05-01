use crate::callback::CALLBACKS;
use crate::properties::{Shape, Properties};
use crate::color::{Pixel, Rgba};

use super::{NodeId, Render, View};

pub fn button() -> Button { Button::new() }

pub struct Button {
    id: NodeId,
    properties: Properties,
}

impl Button {
    pub fn new() -> Self {
        let id = NodeId::new();
        let properties = Properties::new(Rgba::RED, (200, 50), Shape::RoundedRect, false);
        Self { id, properties }
    }

    pub fn style<F: FnOnce(&mut Properties)>(mut self, f: F) -> Self {
        f(&mut self.properties);
        self
    }

    pub fn on_click<F: Fn() + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cb| cb.insert(self.id(), f));
        self
    }
}

impl Render for Button {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[View]> { None }

    fn pixel(&self) -> Option<Pixel<u8>> { None }

    fn properties(&self) -> &Properties { &self.properties }
}
