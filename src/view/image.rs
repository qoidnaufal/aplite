use std::path::PathBuf;

use crate::context::{Alignment, LayoutCtx};
use crate::renderer::{image_reader, Gfx, Gpu, TextureData};
use crate::shapes::{Shape, ShapeConfig, ShapeKind};
use crate::callback::CALLBACKS;
use crate::Rgb;
use super::{AnyView, Configs, IntoView, NodeId, View};

pub fn image<P: Into<PathBuf>>(src: P) -> Image {
    Image::new(src)
}

pub struct Image {
    id: NodeId,
    src: PathBuf,
}

impl Image {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        let id = NodeId::new();
        Self { id, src: path.into() }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn config(&self, gpu: &Gpu, gfx: &mut Gfx, configs: &mut Configs) {
        let mut config = ShapeConfig::new((300, 300), Rgb::WHITE);
        let pixel = image_reader(&self.src);
        config.adjust_ratio(pixel.aspect_ratio());
        gfx.push_texture(TextureData::new(gpu, pixel), &mut config);
        configs.insert(self.id, config);
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

    fn config(&self, gpu: &Gpu, gfx: &mut Gfx, configs: &mut Configs) {
        self.config(gpu, gfx, configs);
    }

    fn img_src(&self) -> Option<&PathBuf> { Some(&self.src) }

    fn shape_kind(&self) -> ShapeKind { ShapeKind::Rect }

    fn layout(&self, cx: &mut LayoutCtx, config: &mut ShapeConfig) {
        cx.assign_position(config);
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
