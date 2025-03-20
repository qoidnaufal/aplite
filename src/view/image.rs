use std::path::PathBuf;

use crate::context::{Alignment, LayoutCtx};
use crate::renderer::image_reader;
use crate::shapes::{Attributes, Shape, ShapeKind};
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
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[AnyView]> { None }

    fn shape(&self) -> Shape {
        Shape::textured(ShapeKind::Rect)
    }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { Some(&self.data) }

    fn layout(&self, cx: &mut LayoutCtx, attr: &mut Attributes) {
        cx.assign_position(attr);
    }

    fn attribs(&self) -> Attributes {
        let mut attr = Attributes::new((300, 300));
        attr.adjust_ratio(self.data.aspect_ratio());
        attr
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
