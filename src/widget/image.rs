use std::path::Path;

use aplite_types::{ImageData, Unit};

use crate::state::AspectRatio;

use super::Widget;

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

pub struct Image {
    width: Unit,
    height: Unit,
    aspect_ratio: AspectRatio,
    data: ImageData,
}

impl Image {
    pub fn new<F: Fn() -> ImageData + 'static>(image_fn: F) -> Self {
        Self {
            width: Unit::Grow,
            height: Unit::Grow,
            aspect_ratio: AspectRatio::Source,
            data: image_fn(),
        }
    }
}

impl Widget for Image {}
