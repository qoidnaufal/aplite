mod button;
mod image;
mod vstack;
mod hstack;

use std::sync::atomic::{AtomicU64, Ordering};

use crate::context::{Alignment, LayoutCtx};
use crate::storage::WidgetStorage;
use crate::renderer::{Gfx, Gpu};
use crate::shapes::{Attributes, Shape, ShapeKind, Style};
use crate::{Pixel, Rgba};
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

pub trait View {
    fn id(&self) -> NodeId;
    fn shape(&self) -> Shape;
    fn children(&self) -> Option<&[AnyView]>;
    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>>;

    fn padding(&self) -> u32;
    fn spacing(&self) -> u32;
    fn alignment(&self) -> Alignment;
    fn attribs(&self) -> Attributes;
    fn layout(&self, cx: &mut LayoutCtx, attribs: &mut Attributes);

    fn build_tree(&self, storage: &mut WidgetStorage) {
        if let Some(children) = self.children() {
            children.iter().for_each(|child| {
                storage.insert_children(self.id(), child.id());
                storage.insert_parent(child.id(), self.id());
                child.build_tree(storage);
            });
        }
    }

    fn prepare(
        &self,
        gpu: &Gpu,
        gfx: &mut Gfx,
        storage: &mut WidgetStorage,
    ) {
        let node_id = self.id();
        if storage.is_root(node_id) { self.build_tree(storage) }
        let mut shape = self.shape();
        let mut attr = self.attribs();
        self.layout(&mut storage.layout, &mut attr);
        let half = attr.dims / 2;
        let current_pos = attr.pos;
        storage.nodes.push(node_id);
        storage.cached_color.insert(node_id, shape.color.into());
        gfx.push_texture(gpu, self.pixel(), &mut shape);
        gfx.register(shape, &attr, gpu.size());
        storage.attribs.insert(node_id, attr);

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

    fn shape(&self) -> Shape {
        self.0.shape()
    }

    fn children(&self) -> Option<&[AnyView]> { self.0.children() }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { self.0.pixel() }

    fn layout(&self, cx: &mut LayoutCtx, attr: &mut Attributes) {
        self.0.layout(cx, attr);
    }

    fn attribs(&self) -> Attributes {
        self.0.attribs()
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
        let any_view = self().into_any();
        DynView(any_view)
    }
}

pub struct TestTriangleWidget {
    id: NodeId,
    style: Style,
}

impl TestTriangleWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        let style = Style::new(Rgba::RED, (300, 300), ShapeKind::Triangle);
        Self { id, style }
    }

    pub fn style<F: FnMut(&mut Style)>(mut self, mut f: F) -> Self {
        f(&mut self.style);
        self
    }

    pub fn on_hover<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    // pub fn on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
    //     self
    // }

    // pub fn on_drag<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
    //     self
    // }
}

impl View for TestTriangleWidget {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[Box<dyn View>]> { None }

    fn shape(&self) -> Shape { Shape::filled(&self.style) }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut LayoutCtx, attr: &mut Attributes) {
        cx.assign_position(attr);
    }

    fn attribs(&self) -> Attributes {
        Attributes::new(self.style.get_dimensions())
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
    style: Style,
}

impl TestCircleWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        let style = Style::new(Rgba::RED, (300, 300), ShapeKind::Circle);
        Self { id, style }
    }

    pub fn style<F: FnMut(&mut Style)>(mut self, mut f: F) -> Self {
        f(&mut self.style);
        self
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
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[Box<dyn View>]> { None }

    fn shape(&self) -> Shape { Shape::filled(&self.style) }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut LayoutCtx, attr: &mut Attributes) {
        cx.assign_position(attr);
    }

    fn attribs(&self) -> Attributes {
        Attributes::new(self.style.get_dimensions())
    }

    fn padding(&self) -> u32 { 0 }

    fn spacing(&self) -> u32 { 0 }

    fn alignment(&self) -> Alignment { Alignment::Vertical }
}

impl IntoView for TestCircleWidget {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
