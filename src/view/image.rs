use std::path::PathBuf;

use crate::layout::LayoutCtx;
use crate::renderer::image_reader;
use crate::element::{Attributes, Element, Shape, Style};
use crate::callback::CALLBACKS;
use crate::{Pixel, Rgba};
use super::{AnyView, IntoView, NodeId, View};

pub fn image<P: Into<PathBuf>>(src: P) -> Image {
    Image::new(src)
}

pub struct Image {
    id: NodeId,
    data: Pixel<Rgba<u8>>,
    style: Style,
}

impl Image {
    fn new<P: Into<PathBuf>>(path: P) -> Self {
        let id = NodeId::new();
        let data = image_reader(path);
        let aspect_ratio = data.aspect_ratio();
        let mut style = Style::new(Rgba::WHITE, (300, 300), Shape::Rect);
        style.adjust_ratio(aspect_ratio);
        Self { id, data, style }
    }

    pub fn style<F: FnMut(&mut Style)>(mut self, mut f: F) -> Self {
        f(&mut self.style);
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

    fn element(&self) -> Element { Element::textured(&self.style) }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { Some(&self.data) }

    fn layout(&self, cx: &mut LayoutCtx) -> Attributes {
        // cx.insert_attributes(self.id, self.style.dimensions());
        cx.assign_position(&self.id)
    }

    fn style(&self) -> Style {
        self.style
    }
}

impl IntoView for Image {
    type V = Self;
    fn into_view(self) -> Self::V {
        self
    }
}
