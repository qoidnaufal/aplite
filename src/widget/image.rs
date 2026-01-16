use std::path::Path;

use aplite_types::{ImageData, Length};

use crate::context::BuildCx;
use crate::layout::LayoutCx;
use crate::widget::Widget;

pub fn image<F: Fn() -> ImageData + 'static>(image_fn: F) -> Image {
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


pub struct Image {
    data: ImageData,
}

impl Image {
    fn new<F: Fn() -> ImageData + 'static>(image_fn: F) -> Self {
        Self {
            data: image_fn(),
        }
    }
}

pub struct ImageState {
    pub width: Length,
    pub height: Length,
    pub aspect_ratio: AspectRatio,
}

impl ImageState {
    fn new() -> Self {
        Self {
            width: Length::Grow,
            height: Length::Grow,
            aspect_ratio: AspectRatio::Source,
        }
    }
}

impl Widget for Image {
    fn build(&self, cx: &mut BuildCx<'_>) {
        cx.set_state(ImageState::new());
    }

    fn layout(&self, _cx: &mut LayoutCx<'_>) {}
}
