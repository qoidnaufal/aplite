use util::Size;
use crate::callback::CALLBACKS;
use crate::storage::WidgetStorage;
use crate::Rgb;
use crate::shapes::{Shape, ShapeKind};
use super::{AnyView, IntoView, NodeId, View};

pub fn hstack(child_nodes: impl IntoIterator<Item = AnyView>) -> HStack {
    HStack::new(child_nodes)
}

pub struct HStack {
    id: NodeId,
    children: Vec<AnyView>,
}

impl HStack {
    fn new(child_nodes: impl IntoIterator<Item = AnyView>) -> Self {
        let id = NodeId::new();
        let children = child_nodes.into_iter().collect();
        Self { id, children }
    }

    fn id(&self) -> NodeId { self.id }

    fn shape(&self) -> Shape {
        let mut size = Size::new(0, 0);
        if !self.children.is_empty() {
            self.children.iter().for_each(|child| {
                let child_size = child.shape().dimensions;
                size.width += child_size.width;
                size.height = size.height.max(child_size.height + self.padding() * 2);
            });
            let child_len = self.children.len() as u32;
            size.width += self.padding() * 2 + self.spacing() * (child_len - 1);
        } else {
            size = (1, 1).into();
        }
        Shape::filled(Rgb::YELLOW, ShapeKind::FilledRectangle, size)
    }

    // pub fn on_hover<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
    //     self
    // }

    pub fn _on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }

    // pub fn on_drag<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
    //     self
    // }
}

impl View for HStack {
    fn id(&self) -> NodeId { self.id() }

    fn children(&self) -> Option<&[AnyView]> { Some(&self.children) }

    fn shape(&self) -> Shape { self.shape() }

    fn img_src(&self) -> Option<&std::path::PathBuf> { None }

    fn layout(&self, cx: &mut WidgetStorage, shape: &mut Shape) {
        cx.layout.align_horizontally();
        cx.layout.assign_position(shape);
        // let half = self.shape().dimensions / 2;
        // let current_pos = if cx.get_parent(&self.id()).is_some() {
        //     cx.layout.next_child_pos()
        // } else {
        //     cx.layout.next_pos()
        // };
        // shape.pos = current_pos + half;

        // cx.layout.set_next_child_pos(|pos| {
        //     pos.x = current_pos.x + self.padding();
        //     pos.y = current_pos.y + self.padding();
        // });

        // self.children.iter().for_each(|child| {
        //     cx.layout.set_next_child_pos(|pos| {
        //         pos.x += child.shape().dimensions.width + self.spacing();
        //     });
        // });

        // if cx.get_parent(&self.id()).is_some() {
        //     cx.layout.set_next_child_pos(|pos| {
        //         pos.x = current_pos.x;
        //         pos.y = current_pos.y;
        //     });
        // } else {
        //     cx.layout.set_next_pos(|pos| {
        //         pos.y += shape.dimensions.height;
        //     });
        // }
    }

    fn padding(&self) -> u32 { 20 }

    fn spacing(&self) -> u32 { 20 }
}

impl IntoView for HStack {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
