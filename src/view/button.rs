use crate::context::Context;
use crate::properties::{Shape, Properties};
use crate::color::Rgba;

use super::{Render, View};

pub fn button(cx: &mut Context) -> View<Button> { Button::new(cx) }

pub struct Button {
    properties: Properties,
}

impl Button {
    pub fn new(cx: &mut Context) -> View<Self> {
        let properties = Properties::new(Rgba::RED, (200, 50), Shape::RoundedRect, false);
        Self { properties }.render(cx, |_| {})
    }

    pub fn style<F: FnOnce(&mut Properties)>(mut self, f: F) -> Self {
        f(&mut self.properties);
        self
    }
}

impl<'a> View<'a, Button> {
    pub fn on_click<F: Fn() + 'static>(self, f: F) -> Self {
        self.cx.add_callbacks(self.id(), f);
        self
    }
}

impl Render for Button {
    fn properties(&self) -> Properties { self.properties }
}
