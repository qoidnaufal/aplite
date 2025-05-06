use std::path::PathBuf;

use crate::renderer::{image_reader, Shape};
use crate::properties::Properties;
use crate::color::Rgba;
use crate::context::Context;
use super::{IntoView, View};

pub fn image<P>(cx: &mut Context, src: P) -> View<Image> where P: Into<PathBuf> {
    Image::new(cx, src)
}

pub struct Image {
    properties: Properties,
}

impl Image {
    pub fn new<P: Into<PathBuf>>(cx: &mut Context, src: P) -> View<Self> {
        let properties = Properties::new()
            .with_size((300, 300))
            .with_shape(Shape::Rect)
            .with_fill_color(Rgba::WHITE)
            .with_textured(true);
        Self { properties }.into_view(cx, |_| {}).add_pixel(src)
    }
}

impl View<'_, Image> {
    pub fn add_pixel<P: Into<PathBuf>>(self, src: P) -> Self {
        let pixel = image_reader(src);
        self.cx.add_pixel(self.id(), pixel);
        self
    }
}

impl IntoView for Image {
    fn debug_name(&self) -> Option<&'static str> { Some("Image") }
    fn properties(&self) -> Properties { self.properties }
}
