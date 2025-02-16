mod button;
mod image;
mod vstack;
mod hstack;

use std::sync::atomic::{AtomicU64, Ordering};
use std::path::PathBuf;
pub use {
    button::*,
    image::*,
    vstack::*,
    hstack::*,
};

use crate::{renderer::{image_reader, Gfx, Gpu, TextureData}, storage::WidgetStorage};
use crate::shapes::{Shape, ShapeKind};
use crate::color::Rgb;
use crate::callback::CALLBACKS;

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

pub trait View {
    fn id(&self) -> NodeId;
    fn shape(&self) -> Shape;
    fn children(&self) -> Option<&[AnyView]>;
    fn img_src(&self) -> Option<&PathBuf>;

    fn layout(&self, cx: &mut WidgetStorage, shape: &mut Shape);
    fn padding(&self) -> u32;
    fn spacing(&self) -> u32;

    fn prepare(
        &self,
        storage: &mut WidgetStorage,
        gpu: &Gpu,
        gfx: &mut Gfx,
        textures: &mut Vec<TextureData>,
    ) {
        let node_id = self.id();
        let mut shape = self.shape();
        let color = if let Some(src) = self.img_src() {
            image_reader(src)
        } else {
            let rgb: Rgb<u8> = shape.color.into();
            storage.cached_color.insert(node_id, rgb);
            rgb.into()
        };

        // local to render space
        // let x = (shape.pos.x / ws.width - 0.5) * 2.0;
        // let y = (0.5 - shape.pos.y / ws.height) * 2.0;

        self.layout(storage, &mut shape);
        let half = shape.dimensions / 2;
        let current_pos = shape.pos;

        storage.nodes.push(node_id);
        gfx.push(shape, gpu.size());
        textures.push(TextureData::new(&gpu, color));

        if let Some(children) = self.children() {
            let current_alignment = storage.layout.current_alignment();
            storage.layout.set_next_pos(|pos| {
                pos.x = current_pos.x - half.width + self.padding();
                pos.y = current_pos.y - half.height + self.padding();
            });

            children.iter().for_each(|child| {
                storage.insert_children(node_id, child.id());
                storage.insert_parent(child.id(), node_id);
                child.prepare(storage, gpu, gfx, textures);

                let is_vertical = storage.layout.is_aligned_vertically();
                storage.layout.set_next_pos(|pos| {
                    if is_vertical {
                        pos.y += child.shape().dimensions.height + self.spacing();
                    } else {
                        pos.x += child.shape().dimensions.width + self.spacing();
                    }
                });
            });
            storage.layout.set_alignment(current_alignment);
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

    fn shape(&self) -> Shape { self.0.shape() }

    fn children(&self) -> Option<&[AnyView]> { self.0.children() }

    fn img_src(&self) -> Option<&PathBuf> { self.0.img_src() }

    fn layout(&self, cx: &mut WidgetStorage, shape: &mut Shape) {
        self.0.layout(cx, shape);
    }

    fn padding(&self) -> u32 { self.0.padding() }

    fn spacing(&self) -> u32 { self.0.spacing() }
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

    fn shape(&self) -> Shape {
        Shape::filled(Rgb::RED, ShapeKind::FilledTriangle, (500, 500))
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

    fn shape(&self) -> Shape { self.shape() }

    fn img_src(&self) -> Option<&PathBuf> { None }

    fn layout(&self, cx: &mut WidgetStorage, shape: &mut Shape) {
        cx.layout.assign_position(shape);
        // let half = self.shape().dimensions / 2;
        // let current_pos = if cx.get_parent(&self.id()).is_some() {
        //     cx.layout.next_child_pos()
        // } else {
        //     cx.layout.next_pos()
        // };
        // shape.pos = current_pos + half;
    }

    fn padding(&self) -> u32 { 0 }

    fn spacing(&self) -> u32 { 0 }
}

impl IntoView for TestTriangleWidget {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
