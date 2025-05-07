use shared::Rgba;
use crate::context::Context;
use crate::properties::Properties;

use super::{IntoView, View};

pub fn button(cx: &mut Context) -> View<Button> { Button::new(cx) }

pub struct Button {
    properties: Properties,
}

impl Button {
    pub fn new(cx: &mut Context) -> View<Self> {
        let properties = Properties::new()
            .with_size((200, 50))
            .with_fill_color(Rgba::RED);
        Self { properties }.into_view(cx, |_| {})
    }
}

impl View<'_, Button> {
    pub fn on_click<F: Fn() + 'static>(self, f: F) -> Self {
        self.cx.add_callbacks(self.id(), f);
        self
    }
}

impl IntoView for Button {
    fn debug_name(&self) -> Option<&'static str> { Some("Button") }
    fn properties(&self) -> Properties { self.properties }
}
