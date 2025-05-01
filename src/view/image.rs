use std::path::PathBuf;

use crate::renderer::image_reader;
use crate::properties::{Shape, Properties};
use crate::color::{Pixel, Rgba};
use crate::context::Context;
use super::{NodeId, Render, View};

pub fn image<P, F>(src: P, f: F) -> Image
where P: Into<PathBuf>, F: FnOnce(&mut Context) {
    Image::new(src, f)
}

pub struct Image {
    id: NodeId,
    src: PathBuf,
    properties: Properties,
}

impl Image {
    pub fn new<P, F>(path: P, f: F) -> Self
    where
        P: Into<PathBuf>,
        F: FnOnce(&mut Context)
    {
        let id = NodeId::new();
        let src = path.into();
        let properties = Properties::new(Rgba::WHITE, (300, 300), Shape::Rect, true);
        Self { id, src, properties }
    }

    pub fn style<F: FnMut(&mut Properties)>(mut self, mut f: F) -> Self {
        f(&mut self.properties);
        self
    }
}

impl Render for Image {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[View]> { None }

    fn pixel(&self) -> Option<Pixel<u8>> {
        let pixel = image_reader(&self.src);
        Some(pixel)
    }

    fn properties(&self) -> &Properties { &self.properties }
}
