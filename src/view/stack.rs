use aplite_types::Rgba;
use aplite_renderer::Shape;

use crate::prelude::Orientation;
use crate::context::Context;
use crate::context::properties::Properties;

use super::{IntoView, View};

pub fn v_stack<F>(cx: &mut Context, child_view: F) -> View<VStack>
where F: FnOnce(&mut Context),
{
    VStack::new(cx, child_view)
}

pub struct VStack {
    properties: Properties,
}

impl VStack {
    pub fn new<F>(cx: &mut Context, child_view: F) -> View<Self>
    where
        F: FnOnce(&mut Context)
    {
        let properties = Properties::new()
            .with_shape(Shape::Rect)
            .with_fill_color(Rgba::DARK_GRAY);
        Self { properties }.into_view(cx, child_view)
    }
}

impl IntoView for VStack {
    fn debug_name(&self) -> Option<&'static str> { Some("VStack") }
    fn properties(&self) -> Properties { self.properties }
}

pub fn h_stack<F>(cx: &mut Context, child_view: F) -> View<HStack>
where F: FnOnce(&mut Context),
{
    HStack::new(cx, child_view)
}

pub struct HStack {
    properties: Properties,
}

impl HStack {
    pub fn new<F>(cx: &mut Context, child_view: F) -> View<Self>
    where
        F: FnOnce(&mut Context)
    {
        let properties = Properties::new()
            .with_shape(Shape::Rect)
            .with_orientation(Orientation::Horizontal)
            .with_fill_color(Rgba::DARK_GRAY);
        Self { properties }.into_view(cx, child_view)
    }
}

impl IntoView for HStack {
    fn debug_name(&self) -> Option<&'static str> { Some("HStack") }
    fn properties(&self) -> Properties { self.properties }
}
