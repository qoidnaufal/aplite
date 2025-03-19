use std::path::PathBuf;

use crate::context::{Alignment, LayoutCtx};
use crate::renderer::image_reader;
use crate::shapes::{Shape, ShapeKind};
use crate::callback::CALLBACKS;
use crate::{Pixel, Rgba};
use super::{AnyView, IntoView, NodeId, View};

pub fn image<P: Into<PathBuf>>(src: P) -> Image {
    Image::new(src)
}

pub struct Image {
    id: NodeId,
    data: Pixel<Rgba<u8>>,
}

impl Image {
    fn new<P: Into<PathBuf>>(path: P) -> Self {
        let id = NodeId::new();
        let data = image_reader(path);
        Self { id, data }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        // let mut config = ShapeConfig::new((300, 300), Rgb::WHITE);
        // config.adjust_ratio(self.data.aspect_ratio());
        // gfx.push_texture(TextureData::new(gpu, &self.data), &mut config);
        // configs.insert(self.id, config);
        Shape::textured(ShapeKind::Rect, self.data.aspect_ratio())
    }

    // pub fn on_hover<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
    //     self
    // }

    // pub fn on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
    //     self
    // }

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for Image {
    fn id(&self) -> NodeId { self.id() }

    fn children(&self) -> Option<&[AnyView]> { None }

    fn shape(&self) -> Shape {
        self.shape()
    }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { Some(&self.data) }

    fn layout(&self, cx: &mut LayoutCtx, shape: &mut Shape) {
        cx.assign_position(shape);
    }

    fn padding(&self) -> u32 { 0 }

    fn spacing(&self) -> u32 { 0 }

    fn alignment(&self) -> Alignment { Alignment::Vertical }
}

impl IntoView for Image {
    type V = Self;
    fn into_view(self) -> Self::V {
        self
    }
}
