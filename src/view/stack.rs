use crate::color::Rgba;
use crate::prelude::Orientation;
use crate::properties::{Properties, Shape};
use crate::context::Context;

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
        Self {
            properties: Properties::new(Rgba::DARK_GRAY, (1, 1), Shape::Rect, false),
        }.into_view(cx, child_view)
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
        let mut properties = Properties::new(Rgba::DARK_GRAY, (1, 1), Shape::Rect, false);
        properties.set_orientation(Orientation::Horizontal);
        Self {
            properties,
        }.into_view(cx, child_view)
    }
}

impl IntoView for HStack {
    fn debug_name(&self) -> Option<&'static str> { Some("HStack") }
    fn properties(&self) -> Properties { self.properties }
}
