use std::path::Path;

use aplite_types::{Fraction, Rgba, Size};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

impl ImageData {
    pub fn new((width, height): (u32, u32), data: &[u8]) -> Self {
        Self { width, height, bytes: data.to_vec() }
    }

    pub fn aspect_ratio(&self) -> Fraction {
        Size::new(self.width as f32, self.height as f32).aspect_ratio()
    }
}

impl std::ops::Deref for ImageData {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.bytes.as_slice()
    }
}

impl From<Rgba<u8>> for ImageData {
    fn from(rgba: Rgba<u8>) -> Self {
        Self::new((1, 1), &rgba.as_slice())
    }
}
