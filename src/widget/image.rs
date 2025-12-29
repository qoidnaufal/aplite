use std::path::Path;

use aplite_types::{ImageData, Length};
use aplite_renderer::Scene;

use crate::context::Context;
use crate::layout::LayoutCx;
use crate::view::IntoView;
use crate::widget::Widget;

pub fn image<F: Fn() -> ImageData + 'static>(image_fn: F) -> impl IntoView {
    Image::new(image_fn)
}

/// This function will resize the image to 500x500 by default to optimize gpu performance.
/// If you want to have your image bytes fully rendered, consider to use your own function
pub fn image_reader<P: AsRef<Path>>(path: P) -> ImageData {
    use image::imageops::FilterType;
    use image::{GenericImageView, ImageReader};

    let img = ImageReader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .resize_to_fill(500, 500, FilterType::Lanczos3);

    ImageData::new(img.dimensions(), &img.to_rgba8())
}

#[derive(Debug, Clone, Copy)]
pub enum AspectRatio {
    Defined(u8, u8),
    Source,
    Undefined,
}


struct Image {
    width: Length,
    height: Length,
    aspect_ratio: AspectRatio,
    data: ImageData,
}

impl Image {
    fn new<F: Fn() -> ImageData + 'static>(image_fn: F) -> Self {
        Self {
            width: Length::Grow,
            height: Length::Grow,
            aspect_ratio: AspectRatio::Source,
            data: image_fn(),
        }
    }
}

impl Widget for Image {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) {
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

impl IntoView for Image {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}
