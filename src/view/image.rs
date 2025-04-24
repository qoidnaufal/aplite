use std::path::PathBuf;

use crate::renderer::image_reader;
use crate::element::Element;
use crate::properties::{Shape, Properties};
use crate::callback::CALLBACKS;
use crate::color::{Pixel, Rgba};
use crate::context::Context;
use super::{AnyView, IntoView, NodeId, View};

pub fn image<P: Into<PathBuf>>(src: P) -> Image {
    Image::new(src)
}

pub struct Image {
    id: NodeId,
    pixel: Pixel<Rgba<u8>>,
    inner: Properties,
}

impl Image {
    fn new<P: Into<PathBuf>>(path: P) -> Self {
        let id = NodeId::new();
        let pixel = image_reader(path);
        let aspect_ratio = pixel.aspect_ratio();
        let mut inner = Properties::new(Rgba::WHITE, (300, 300), Shape::Rect, true);
        inner.adjust_ratio(aspect_ratio);
        Self { id, pixel, inner }
    }

    pub fn style<F: FnMut(&mut Properties)>(mut self, mut f: F) -> Self {
        f(&mut self.inner);
        self
    }

    pub fn on_hover<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    pub fn on_click<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }

    pub fn on_drag<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for Image {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[AnyView]> { None }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { Some(&self.pixel) }

    fn layout(&self, cx: &mut Context) { cx.assign_position(&self.id) }

    fn properties(&self) -> &Properties { &self.inner }
}

impl IntoView for Image {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
