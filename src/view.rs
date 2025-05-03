mod button;
mod image;
mod stack;

use std::marker::PhantomData;

use crate::properties::{Orientation, Shape, Properties, HAlign, VAlign};
use crate::context::Context;
use crate::color::Rgba;
use crate::tree::NodeId;

pub use {
    button::*,
    image::*,
    stack::*,
};

pub trait Render: Sized {
    fn debug_name<'a>(&self) -> &'a str;
    fn properties(&self) -> Properties;

    // fn children(&self) -> Option<&[View<Self>]>;

    // fn layout(&self, cx: &mut Context, node_id: &NodeId) {
    //     if self.children().is_some() {
    //         cx.set_orientation(node_id);
    //         cx.set_alignment(node_id);
    //         cx.set_spacing(node_id);
    //         cx.set_padding(node_id);
    //     }
    //     cx.assign_position(node_id);
    // }

    // fn calculate_size(&self, cx: &mut Context) {
    //     let prop = self.properties();
    //     let padding = prop.padding();
    //     let mut size = prop.size();

    //     if let Some(children) = self.children() {
    //         children.iter().for_each(|child| {
    //             child.calculate_size(cx);
    //             let child_size = cx.get_node_data(&child.id()).size();
    //             match prop.orientation() {
    //                 Orientation::Vertical => {
    //                     size.height += child_size.height;
    //                     size.width = size.width.max(child_size.width + padding.horizontal());
    //                 }
    //                 Orientation::Horizontal => {
    //                     size.height = size.height.max(child_size.height + padding.vertical());
    //                     size.width += child_size.width - 1;
    //                 }
    //             }
    //         });
    //         let child_len = children.len() as u32;
    //         let stretch = prop.spacing() * (child_len - 1);
    //         match prop.orientation() {
    //             Orientation::Vertical => {
    //                 size.height += padding.vertical() + stretch;
    //             },
    //             Orientation::Horizontal => {
    //                 size.width += padding.horizontal() + stretch;
    //             },
    //         }
    //     }

    //     let final_size = size
    //         .max(prop.min_width(), prop.min_height())
    //         .min(prop.max_width(), prop.max_height());

    //     let properties = cx.get_node_data_mut(&self.id());
    //     properties.set_size(final_size);
    // }

    fn render<F>(self, cx: &mut Context, child_fn: F) -> View<Self>
    where F: FnOnce(&mut Context),
    {
        let node_id = cx.create_entity();
        let parent = cx.current_entity();
        cx.insert(node_id, parent, self.properties());
        cx.set_current_entity(Some(node_id));

        // let name = self.debug_name();
        // eprintln!("{name}: node_id: {node_id:?} | parent: {parent:?}");

        child_fn(cx);
        cx.calculate_size(&node_id);
        // self.layout(cx, &node_id);
        // let properties = cx.get_node_data(&node_id);

        // if let Some(child) = child_view {
        //     child(cx);
            // let padding = properties.padding();
            // let current_pos = properties.pos();
            // let current_half = properties.size() / 2;

            // // FIXME: consider alignment
            // cx.set_next_pos(|pos| {
            //     pos.x = current_pos.x - current_half.width + padding.left();
            //     pos.y = current_pos.y - current_half.height + padding.top();
            // });

            // // children.iter().for_each(|child| child.render(cx));

            // if let Some(parent_idx) = cx.tree.get_parent(&node_id).copied() {
            //     cx.reset_to_parent(&parent_idx, current_pos, current_half);
            // }
        // }

        cx.set_current_entity(parent);

        View::new(node_id, parent, cx)
    }
}

pub struct View<'a, R: Render> {
    entity: NodeId,
    parent: Option<NodeId>,
    cx: &'a mut Context,
    inner: PhantomData<R>,
}

impl<'a, R: Render> View<'a, R> {
    fn new(entity: NodeId, parent: Option<NodeId>, cx: &'a mut Context) -> Self {
        Self {
            entity,
            parent,
            cx,
            inner: PhantomData,
        }
    }

    pub(crate) fn id(&self) -> NodeId {
        self.entity
    }

    pub fn style<P: Fn(&mut Properties) + 'static>(self, f: P) -> Self {
        let prop = self.cx.get_node_data_mut(&self.id());
        f(prop);
        self.cx.add_style_fn(self.id(), f);
        self
    }
}

#[derive(Clone)]
pub struct TestTriangleWidget {
    properties: Properties,
}

impl TestTriangleWidget {
    pub fn new(cx: &mut Context) -> View<Self> {
        let properties = Properties::new(Rgba::RED, (300, 300), Shape::Triangle, false);
        Self { properties }.render(cx, |_| {})
    }
}

impl Render for TestTriangleWidget {
    fn debug_name<'a>(&self) -> &'a str { "TestTriangleWidget" }
    fn properties(&self) -> Properties { self.properties }
}

#[derive(Clone)]
pub struct TestCircleWidget {
    properties: Properties,
}

impl TestCircleWidget {
    pub fn new(cx: &mut Context) -> View<Self> {
        let properties = Properties::new(Rgba::RED, (300, 300), Shape::Circle, false);
        Self { properties }.render(cx, |_| {})
    }
}

impl Render for TestCircleWidget {
    fn debug_name<'a>(&self) -> &'a str { "TestCircleWidget" }
    fn properties(&self) -> Properties { self.properties }
}
