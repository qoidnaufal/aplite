use shared::Rgba;

use crate::{image_data::ImageData, properties::Properties};
use crate::context::Context;
use crate::renderer::element::Shape;
use crate::properties::AspectRatio;

use super::{IntoView, View};

pub fn image<F: Fn() -> ImageData + 'static>(cx: &mut Context, f: F) -> View<Image> {
    Image::new(cx, f)
}

pub struct Image {
    properties: Properties,
}

impl Image {
    pub fn new<F: Fn() -> ImageData + 'static>(cx: &mut Context, f: F) -> View<Self> {
        let properties = Properties::new()
            .with_size((100, 100))
            .with_shape(Shape::Rect)
            .with_fill_color(Rgba::WHITE)
            .with_textured(true);
        Self { properties }.into_view(cx, |_| {}).add_data(f)
    }
}

impl View<'_, Image> {
    fn add_data<F: Fn() -> ImageData + 'static>(self, image_fn: F) -> Self {
        self.cx.add_image(self.id(), image_fn);
        self
    }

    pub fn with_aspect_ratio(self, aspect_ratio: AspectRatio) -> Self {
        self.cx.get_node_data_mut(&self.id()).set_image_aspect_ratio(aspect_ratio);
        self
    }
}

impl IntoView for Image {
    fn debug_name(&self) -> Option<&'static str> { Some("Image") }
    fn properties(&self) -> Properties { self.properties }
}
