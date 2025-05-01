mod button;
mod image;
mod stack;

use crate::properties::{Orientation, Shape, Properties, HAlign, VAlign};
use crate::context::Context;
use crate::tree::NodeId;
use crate::renderer::{Gfx, Gpu};
use crate::color::{Pixel, Rgba};

pub use {
    button::*,
    image::*,
    stack::*,
};

pub trait IntoView: Sized {
    fn into_view(self) -> View;
}

impl<T: Render + 'static> IntoView for T {
    fn into_view(self) -> View { View::new(self) }
}

pub trait Render {
    fn id(&self) -> NodeId;
    fn children(&self) -> Option<&[View]>;
    fn pixel(&self) -> Option<Pixel<u8>>;
    fn properties(&self) -> &Properties;

    fn layout(&self, cx: &mut Context, node_id: &NodeId) {
        if self.children().is_some() {
            cx.set_orientation(node_id);
            cx.set_alignment(node_id);
            cx.set_spacing(node_id);
            cx.set_padding(node_id);
        }
        cx.assign_position(node_id);
    }

    fn calculate_size(&self, cx: &mut Context) {
        let prop = self.properties();
        let padding = prop.padding();
        let mut size = prop.size();

        if let Some(children) = self.children() {
            children.iter().for_each(|child| {
                child.calculate_size(cx);
                let child_size = cx.get_node_data(&child.id()).size();
                match prop.orientation() {
                    Orientation::Vertical => {
                        size.height += child_size.height;
                        size.width = size.width.max(child_size.width + padding.horizontal());
                    }
                    Orientation::Horizontal => {
                        size.height = size.height.max(child_size.height + padding.vertical());
                        size.width += child_size.width - 1;
                    }
                }
            });
            let child_len = children.len() as u32;
            let stretch = prop.spacing() * (child_len - 1);
            match prop.orientation() {
                Orientation::Vertical => {
                    size.height += padding.vertical() + stretch;
                },
                Orientation::Horizontal => {
                    size.width += padding.horizontal() + stretch;
                },
            }
        }

        let final_size = size
            .max(prop.min_width(), prop.min_height())
            .min(prop.max_width(), prop.max_height());

        let properties = cx.get_node_data_mut(&self.id());
        properties.set_size(final_size);
    }

    fn prepare(&self, cx: &mut Context, parent_id: Option<NodeId>) {
        let node_id = cx.create_entity();
        cx.insert(node_id, parent_id, *self.properties(), self.pixel());
        if let Some(children) = self.children() {
            children.iter().for_each(|child_view| child_view.prepare(cx, Some(node_id)));
        }
    }

    fn render(&self, cx: &mut Context) {
        let node_id = cx.create_entity();
        cx.insert(node_id, None, *self.properties(), self.pixel());
        // if cx.tree.is_empty() {
        //     self.prepare(cx, None);
        //     self.calculate_size(cx);
        // }
        self.layout(cx, &node_id);
        let properties = cx.get_node_data(&node_id);

        if let Some(children) = self.children() {
            let padding = properties.padding();
            let current_pos = properties.pos();
            let current_half = properties.size() / 2;

            // FIXME: consider alignment
            cx.set_next_pos(|pos| {
                pos.x = current_pos.x - current_half.width + padding.left();
                pos.y = current_pos.y - current_half.height + padding.top();
            });

            children.iter().for_each(|child| child.render(cx));

            if let Some(parent_idx) = cx.tree.get_parent(&node_id).copied() {
                cx.reset_to_parent(&parent_idx, current_pos, current_half);
            }
        }
    }
}

pub struct View {
    inner: Box<dyn Render>,
}

impl View {
    fn new(inner: impl Render + 'static) -> Self {
        Self { inner: Box::new(inner) }
    }
}

impl Render for View {
    fn id(&self) -> NodeId { self.inner.id() }

    fn children(&self) -> Option<&[View]> { self.inner.children() }

    fn pixel(&self) -> Option<Pixel<u8>> { self.inner.pixel() }

    fn properties(&self) -> &Properties { self.inner.properties() }
}

pub struct TestTriangleWidget {
    id: NodeId,
    properties: Properties,
}

impl TestTriangleWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        let properties = Properties::new(Rgba::RED, (300, 300), Shape::Triangle, false);
        Self { id, properties }
    }

    pub fn style<F: FnOnce(&mut Properties)>(mut self, f: F) -> Self {
        f(&mut self.properties);
        self
    }
}

impl Render for TestTriangleWidget {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[View]> { None }

    fn pixel(&self) -> Option<Pixel<u8>> { None }

    fn properties(&self) -> &Properties { &self.properties }
}

pub struct TestCircleWidget {
    id: NodeId,
    properties: Properties,
}

impl TestCircleWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        let properties = Properties::new(Rgba::RED, (300, 300), Shape::Circle, false);
        Self { id, properties }
    }

    pub fn style<F: FnMut(&mut Properties)>(mut self, mut f: F) -> Self {
        f(&mut self.properties);
        self
    }
}

impl Render for TestCircleWidget {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[View]> { None }

    fn pixel(&self) -> Option<Pixel<u8>> { None }

    fn properties(&self) -> &Properties { &self.properties }
}
