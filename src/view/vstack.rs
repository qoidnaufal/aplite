use util::Size;

use crate::callback::CALLBACKS;
use crate::context::{Alignment, LayoutCtx};
use crate::renderer::{Gfx, Gpu};
use crate::Rgb;
use crate::shapes::{Shape, ShapeConfig, ShapeKind};

use super::{AnyView, Configs, IntoView, NodeId, View};

pub fn vstack(child_nodes: impl IntoIterator<Item = AnyView>) -> VStack {
    VStack::new(child_nodes)
}

pub struct VStack {
    id: NodeId,
    children: Vec<Box<dyn View>>,
}

impl VStack {
    fn new(child_nodes: impl IntoIterator<Item = AnyView>) -> Self {
        let id = NodeId::new();
        let children = child_nodes.into_iter().collect();
        Self { id, children }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn config(&self, gpu: &Gpu, gfx: &mut Gfx, configs: &mut Configs) {
        let mut size = Size::new(0, 0);
        if !self.children.is_empty() {
            self.children.iter().for_each(|child| {
                child.config(gpu, gfx, configs);
                let child_size = configs.get(&child.id()).unwrap().dims;
                size.height += child_size.height;
                size.width = size.width.max(child_size.width + self.padding() * 2);
            });
            let child_len = self.children.len() as u32;
            size.height += self.padding() * 2 + self.spacing() * (child_len - 1);
        } else { size = (1, 1).into() }
        let config = ShapeConfig::new(size, Rgb::DARK_GRAY);
        configs.insert(self.id, config);
        // let shape = Shape::filled(Rgb::DARK_GRAY, ShapeKind::Rect, size);
        // gfx.push(shape, gpu.size());
    }

    pub fn on_hover<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    // pub fn on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
    //     self
    // }

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for VStack {
    fn id(&self) -> NodeId { self.id() }

    fn children(&self) -> Option<&[AnyView]> { Some(&self.children) }

    fn config(&self, gpu: &Gpu, gfx: &mut Gfx, configs: &mut Configs) {
        self.config(gpu, gfx, configs);
    }

    fn img_src(&self) -> Option<&std::path::PathBuf> { None }

    fn shape_kind(&self) -> ShapeKind { ShapeKind::Rect }

    fn layout(&self, cx: &mut LayoutCtx, shape: &mut ShapeConfig) {
        cx.align_vertically();
        cx.assign_position(shape);
    }

    fn padding(&self) -> u32 { 20 }

    fn spacing(&self) -> u32 { 20 }

    fn alignment(&self) -> Alignment { Alignment::Vertical }
}

impl IntoView for VStack {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
