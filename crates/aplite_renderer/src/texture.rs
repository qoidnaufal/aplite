use std::path::Path;

mod atlas;
mod image_data;
mod texture_data;

pub(crate) use atlas::{Atlas, AtlasInfo};
pub(crate) use texture_data::TextureData;
pub use image_data::ImageData;

/// This function will resize the image to 450x450 by default to optimize gpu performance.
/// If you want to have your image bytes fully rendered, consider to use your own function
pub fn image_reader<P: AsRef<Path>>(path: P) -> ImageData {
    use image::imageops::FilterType;
    use image::{GenericImageView, ImageReader};

    let img = ImageReader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .resize_to_fill(450, 450, FilterType::Lanczos3);

    ImageData::new(img.dimensions(), &img.to_rgba8())
}
