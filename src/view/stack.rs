use crate::color::Rgba;
use crate::properties::{Properties, Shape};
use crate::context::Context;

use super::{Render, View};

pub fn stack<F>(cx: &mut Context, child_view: F) -> View<Stack>
where F: FnOnce(&mut Context),
{
    Stack::new(cx, child_view)
}

pub struct Stack {
    properties: Properties,
}

impl Stack {
    pub fn new<F>(cx: &mut Context, child_view: F) -> View<Self>
    where
        F: FnOnce(&mut Context)
    {
        Self {
            properties: Properties::new(Rgba::DARK_GRAY, (1, 1), Shape::Rect, false),
        }.render(cx, child_view)
    }
}

impl Render for Stack {
    fn properties(&self) -> Properties { self.properties }
}
