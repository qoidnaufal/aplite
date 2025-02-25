mod button;
mod image;
mod vstack;
mod hstack;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::path::PathBuf;

use crate::context::{Alignment, LayoutCtx};
use crate::storage::WidgetStorage;
use crate::renderer::{Gfx, Gpu};
use crate::shapes::{Shape, ShapeConfig, ShapeKind};
use crate::Rgb;
use crate::callback::CALLBACKS;

pub use {
    button::*,
    image::*,
    vstack::*,
    hstack::*,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    fn new() -> Self {
        static NODE_ID: AtomicU64 = AtomicU64::new(0);
        Self(NODE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub type AnyView = Box<dyn View>;

impl std::fmt::Debug for AnyView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.id())
    }
}

pub type Configs = HashMap<NodeId, ShapeConfig>;

pub trait View {
    fn id(&self) -> NodeId;
    fn config(&self, gpu: &Gpu, gfx: &mut Gfx, configs: &mut Configs);
    fn children(&self) -> Option<&[AnyView]>;
    fn img_src(&self) -> Option<&PathBuf>;
    fn shape_kind(&self) -> ShapeKind;

    fn build_tree(&self, storage: &mut WidgetStorage) {
        // storage.nodes.push(self.id());
        if let Some(children) = self.children() {
            children.iter().for_each(|child| {
                storage.insert_children(self.id(), child.id());
                storage.insert_parent(child.id(), self.id());
                child.build_tree(storage);
            });
        }
    }

    fn layout(&self, cx: &mut LayoutCtx, shape: &mut ShapeConfig);
    fn padding(&self) -> u32;
    fn spacing(&self) -> u32;
    fn alignment(&self) -> Alignment;

    fn prepare(
        &self,
        gpu: &Gpu,
        gfx: &mut Gfx,
        storage: &mut WidgetStorage,
    ) {
        let node_id = self.id();
        if storage.is_root(node_id) {
            self.build_tree(storage);
            // image widget need to consider the aspect ratio
            self.config(gpu, gfx, &mut storage.configs);
        }
        let config = storage.configs.get_mut(&node_id).unwrap();
        self.layout(&mut storage.layout, config);
        let half = config.dims / 2;
        let current_pos = config.pos;

        storage.nodes.push(node_id);
        gfx.push(config, gpu.size(), self.shape_kind());

        if let Some(children) = self.children() {
            storage.layout.insert_alignment(node_id, self.alignment());
            storage.layout.set_next_pos(|pos| {
                pos.x = current_pos.x - half.width + self.padding();
                pos.y = current_pos.y - half.height + self.padding();
            });
            storage.layout.set_spacing(self.spacing());
            storage.layout.set_padding(self.padding());

            children.iter().for_each(|child| {
                child.prepare(gpu, gfx, storage);
            });

            if let Some(parent_id) = storage.get_parent(node_id) {
                storage.layout.reset_to_parent(*parent_id, current_pos, half);
            }
        }

    }
}

pub trait IntoView: Sized {
    type V: View + 'static;
    fn into_view(self) -> Self::V;
    fn into_any(self) -> AnyView { Box::new(self.into_view()) }
}

pub struct DynView(AnyView);

impl View for DynView {
    fn id(&self) -> NodeId { self.0.id() }

    fn config(&self, gpu: &Gpu, gfx: &mut Gfx, configs: &mut Configs) {
        self.0.config(gpu, gfx, configs)
    }

    fn children(&self) -> Option<&[AnyView]> { self.0.children() }

    fn img_src(&self) -> Option<&PathBuf> { self.0.img_src() }

    fn shape_kind(&self) -> ShapeKind { self.0.shape_kind() }

    fn layout(&self, cx: &mut LayoutCtx, config: &mut ShapeConfig) {
        self.0.layout(cx, config);
    }

    fn padding(&self) -> u32 { self.0.padding() }

    fn spacing(&self) -> u32 { self.0.spacing() }

    fn alignment(&self) -> Alignment { self.0.alignment() }
}

impl<F, IV> IntoView for F
where
    F: Fn() -> IV + 'static,
    IV: IntoView + 'static
{
    type V = DynView;
    fn into_view(self) -> Self::V {
        let a = self().into_any();
        DynView(a)
    }
}

pub struct TestTriangleWidget {
    id: NodeId,
}

impl TestTriangleWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId { self.id }

    fn config(&self, _gpu: &Gpu, _gfx: &mut Gfx, configs: &mut Configs) {
        let config = ShapeConfig::new((300, 300), Rgb::RED);
        configs.insert(self.id, config);
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

impl View for TestTriangleWidget {
    fn id(&self) -> NodeId { self.id() }

    fn children(&self) -> Option<&[Box<dyn View>]> { None }

    fn config(&self, gpu: &Gpu, gfx: &mut Gfx, configs: &mut Configs) {
        self.config(gpu, gfx, configs);
    }

    fn img_src(&self) -> Option<&PathBuf> { None }

    fn shape_kind(&self) -> ShapeKind { ShapeKind::Triangle }

    fn layout(&self, cx: &mut LayoutCtx, config: &mut ShapeConfig) {
        cx.assign_position(config);
    }

    fn padding(&self) -> u32 { 0 }

    fn spacing(&self) -> u32 { 0 }

    fn alignment(&self) -> Alignment { Alignment::Vertical }
}

impl IntoView for TestTriangleWidget {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}

pub struct TestCircleWidget {
    id: NodeId,
}

impl TestCircleWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId { self.id }

    fn config(&self, _gpu: &Gpu, _gfx: &mut Gfx, configs: &mut Configs) {
        let config = ShapeConfig::new((300, 300), Rgb::RED);
        configs.insert(self.id, config);
        // Shape::filled(Rgb::RED, ShapeKind::Circle, (500, 500))
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

impl View for TestCircleWidget {
    fn id(&self) -> NodeId { self.id() }

    fn children(&self) -> Option<&[Box<dyn View>]> { None }

    fn config(&self, gpu: &Gpu, gfx: &mut Gfx, configs: &mut Configs) {
        self.config(gpu, gfx, configs);
    }

    fn img_src(&self) -> Option<&PathBuf> { None }

    fn shape_kind(&self) -> ShapeKind { ShapeKind::Circle }

    fn layout(&self, cx: &mut LayoutCtx, config: &mut ShapeConfig) {
        cx.assign_position(config);
    }

    fn padding(&self) -> u32 { 0 }

    fn spacing(&self) -> u32 { 0 }

    fn alignment(&self) -> Alignment { Alignment::Vertical }
}

impl IntoView for TestCircleWidget {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
